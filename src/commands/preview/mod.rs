mod fiddle_messenger;
use fiddle_messenger::*;

mod http_method;
pub use http_method::HTTPMethod;

mod request_payload;
pub use request_payload::RequestPayload;

mod upload;
pub use upload::upload;

use std::process::Command;
use std::sync::mpsc::channel;
use std::thread;

use log::info;
use url::Url;
use ws::{Sender, WebSocket};

use crate::build;
use crate::http;
use crate::settings::global_user::GlobalUser;
use crate::settings::toml::Target;
use crate::terminal::message;
use crate::watch::watch_and_build;

pub fn preview(
    mut target: Target,
    user: Option<GlobalUser>,
    method: HTTPMethod,
    url: Url,
    body: Option<String>,
    livereload: bool,
    verbose: bool,
    headless: bool,
) -> Result<(), failure::Error> {
    build(&target)?;

    let sites_preview: bool = target.site.is_some();

    let script_id = upload(&mut target, user.as_ref(), sites_preview, verbose)?;

    let request_payload = RequestPayload::create(method, url, body);

    let session = &request_payload.session;
    let browser_url = &request_payload.browser_url;

    if livereload {
        // explicitly use 127.0.0.1, since localhost can resolve to 2 addresses
        let server = WebSocket::new(|out| FiddleMessageServer { out })?.bind("127.0.0.1:0")?;

        let ws_port = server.local_addr()?.port();

        info!("Opened websocket server on port {}", ws_port);

        if !headless {
            open_browser(&format!(
                "https://cloudflareworkers.com/?wrangler_session_id={0}&wrangler_ws_port={1}&hide_editor#{2}:{3}",
                session, ws_port, script_id, browser_url
            ))?;
        }

        // Make a the initial request to the URL
        client_request(&request_payload, &script_id, &sites_preview);

        let broadcaster = server.broadcaster();
        thread::spawn(move || server.run());
        watch_for_changes(
            target,
            user.as_ref(),
            broadcaster,
            verbose,
            headless,
            request_payload,
        )?;
    } else {
        if !headless {
            open_browser(&format!(
                "https://cloudflareworkers.com/?hide_editor#{0}:{1}",
                script_id, browser_url
            ))?;
        }

        client_request(&request_payload, &script_id, &sites_preview);
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

fn client_request(payload: &RequestPayload, script_id: &String, sites_preview: &bool) {
    let client = http::client();

    let method = &payload.method;
    let url = &payload.service_url;
    let body = &payload.body;

    let cookie = payload.cookie(script_id);

    let worker_res = match method {
        HTTPMethod::Get => get(&url, &cookie, &client).unwrap(),
        HTTPMethod::Post => post(&url, &cookie, &body, &client).unwrap(),
    };

    let msg = if *sites_preview {
        "Your Worker is a Workers Site, please preview it in browser window.".to_string()
    } else {
        format!("Your Worker responded with: {}", worker_res)
    };
    message::preview(&msg);
}

fn get(
    url: &String,
    cookie: &String,
    client: &reqwest::blocking::Client,
) -> Result<String, failure::Error> {
    let res = client.get(url).header("Cookie", cookie).send();
    Ok(res?.text()?)
}

fn post(
    url: &String,
    cookie: &String,
    body: &Option<String>,
    client: &reqwest::blocking::Client,
) -> Result<String, failure::Error> {
    let res = match body {
        Some(s) => client
            .post(url)
            .header("Cookie", cookie)
            .body(format!("{}", s))
            .send(),
        None => client.post(url).header("Cookie", cookie).send(),
    };
    let msg = format!("POST {}", url);
    message::preview(&msg);
    Ok(res?.text()?)
}

fn watch_for_changes(
    mut target: Target,
    user: Option<&GlobalUser>,
    broadcaster: Sender,
    verbose: bool,
    headless: bool,
    request_payload: RequestPayload,
) -> Result<(), failure::Error> {
    let sites_preview: bool = target.site.is_some();

    let (tx, rx) = channel();
    watch_and_build(&target, Some(tx))?;

    while let Ok(_) = rx.recv() {
        if let Ok(new_id) = upload(&mut target, user, sites_preview, verbose) {
            let script_id = format!("{}", new_id);

            let msg = FiddleMessage {
                session_id: request_payload.session.clone(),
                data: FiddleMessageData::LiveReload {
                    new_id: new_id.clone(),
                },
            };

            if !headless {
                match broadcaster.send(serde_json::to_string(&msg)?) {
                    Ok(_) => {
                        message::preview("Updated preview with changes");
                    }
                    Err(_e) => message::user_error("communication with preview failed"),
                }
            }

            client_request(&request_payload, &script_id, &sites_preview);
        }
    }

    broadcaster.shutdown()?;

    Ok(())
}
