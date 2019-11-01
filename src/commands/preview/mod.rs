use std::process::Command;

mod fiddle_messenger;
use fiddle_messenger::*;

mod http_method;
pub use http_method::HTTPMethod;

mod upload;
use upload::upload;

use crate::commands;

use uuid::Uuid;

use log::info;

use crate::http;
use crate::settings::global_user::GlobalUser;
use crate::settings::target::Target;
use crate::terminal::message;

use std::sync::mpsc::channel;
use std::thread;
use ws::{Sender, WebSocket};

// Using this instead of just `https://cloudflareworkers.com` returns just the worker response to the CLI
const PREVIEW_ADDRESS: &str = "https://00000000000000000000000000000000.cloudflareworkers.com";

pub fn preview(
    mut target: Target,
    user: Option<GlobalUser>,
    method: HTTPMethod,
    body: Option<String>,
    livereload: bool,
    verbose: bool,
    browser: bool,
) -> Result<(), failure::Error> {
    commands::build(&target)?;

    let sites_preview: bool = target.site.is_some();

    let script_id = upload(&mut target, user.as_ref(), sites_preview, verbose)?;

    let session = Uuid::new_v4().to_simple();
    let preview_host = "example.com";
    let https = true;
    let https_str = if https { "https://" } else { "http://" };

    if livereload {
        let server = WebSocket::new(|out| FiddleMessageServer { out })?.bind("127.0.0.1:0")?; //explicitly use 127.0.0.1, since localhost can resolve to 2 addresses

        let ws_port = server.local_addr()?.port();

        info!("Opened websocket server on port {}", ws_port);

        if browser {
            open_browser(&format!(
            "https://cloudflareworkers.com/?wrangler_session_id={0}&wrangler_ws_port={1}&hide_editor#{2}:{3}{4}",
            &session.to_string(), ws_port, script_id, https_str, preview_host,
        ))?;
        }

        //don't do initial GET + POST with livereload as the expected behavior is unclear.

        let broadcaster = server.broadcaster();
        thread::spawn(move || server.run());
        watch_for_changes(
            target,
            user.as_ref(),
            session.to_string(),
            broadcaster,
            verbose,
        )?;
    } else {
        if browser {
            open_browser(&format!(
                "https://cloudflareworkers.com/?hide_editor#{0}:{1}{2}",
                script_id, https_str, preview_host
            ))?;
        }

        let cookie = format!(
            "__ew_fiddle_preview={}{}{}{}",
            script_id, session, https as u8, preview_host
        );

        let client = http::client(None);

        let worker_res = match method {
            HTTPMethod::Get => get(cookie, &client)?,
            HTTPMethod::Post => post(cookie, &client, body)?,
        };
        let msg = if sites_preview {
            "Your Worker is a Workers Site, please preview it in browser window.".to_string()
        } else {
            format!("Your Worker responded with: {}", worker_res)
        };
        message::preview(&msg);
    }

    Ok(())
}

fn open_browser(url: &str) -> Result<(), failure::Error> {
    let _output = if cfg!(target_os = "windows") {
        let url_escaped = url.replace("&", "^&");
        let windows_cmd = format!("start {}", url_escaped);
        Command::new("cmd").args(&["/C", &windows_cmd]).output()?
    } else if cfg!(target_os = "linux") {
        let linux_cmd = format!(r#"xdg-open "{}""#, url);
        Command::new("sh").arg("-c").arg(&linux_cmd).output()?
    } else {
        let mac_cmd = format!(r#"open "{}""#, url);
        Command::new("sh").arg("-c").arg(&mac_cmd).output()?
    };

    Ok(())
}

fn get(cookie: String, client: &reqwest::Client) -> Result<String, failure::Error> {
    let res = client.get(PREVIEW_ADDRESS).header("Cookie", cookie).send();
    Ok(res?.text()?)
}

fn post(
    cookie: String,
    client: &reqwest::Client,
    body: Option<String>,
) -> Result<String, failure::Error> {
    let res = match body {
        Some(s) => client
            .post(PREVIEW_ADDRESS)
            .header("Cookie", cookie)
            .body(s)
            .send(),
        None => client.post(PREVIEW_ADDRESS).header("Cookie", cookie).send(),
    };
    let msg = format!("POST {}", PREVIEW_ADDRESS);
    message::preview(&msg);
    Ok(res?.text()?)
}

fn watch_for_changes(
    mut target: Target,
    user: Option<&GlobalUser>,
    session_id: String,
    broadcaster: Sender,
    verbose: bool,
) -> Result<(), failure::Error> {
    let sites_preview: bool = target.site.is_some();

    let (tx, rx) = channel();
    commands::watch_and_build(&target, Some(tx))?;

    while let Ok(_e) = rx.recv() {
        commands::build(&target)?;

        if let Ok(new_id) = upload(&mut target, user, sites_preview, verbose) {
            let msg = FiddleMessage {
                session_id: session_id.clone(),
                data: FiddleMessageData::LiveReload { new_id },
            };

            match broadcaster.send(serde_json::to_string(&msg)?) {
                Ok(_) => {
                    message::preview("Updated preview with changes");
                }
                Err(_e) => message::user_error("communication with preview failed"),
            }
        }
    }

    broadcaster.shutdown()?;

    Ok(())
}
