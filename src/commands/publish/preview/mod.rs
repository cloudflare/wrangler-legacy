use std::process::Command;

mod fiddle_messenger;
use fiddle_messenger::*;

mod http_method;
pub use http_method::HTTPMethod;

use crate::commands::build;
use crate::commands::watch_and_build;

use serde::Deserialize;
use uuid::Uuid;

use super::upload_form::build_script_upload_form;
use crate::http;
use crate::settings::project::Project;
use crate::terminal::message;

use std::sync::mpsc::channel;
use std::thread;
use ws::{Sender, WebSocket};

pub fn preview(
    project: &Project,
    method: Result<HTTPMethod, failure::Error>,
    body: Option<String>,
    livereload: bool,
) -> Result<(), failure::Error> {
    build(&project)?; //do the initial build

    let session = Uuid::new_v4().to_simple();

    let preview_host = "example.com";
    let https = true;
    let https_str = if https { "https://" } else { "http://" };
    let script_id = &upload_and_get_id(project)?;

    let preview_address = "https://00000000000000000000000000000000.cloudflareworkers.com";
    let cookie = format!(
        "__ew_fiddle_preview={}{}{}{}",
        script_id, session, https as u8, preview_host
    );

    let method = method.unwrap_or_default();

    let client = http::client();
    let worker_res = match method {
        HTTPMethod::Get => get(preview_address, cookie, client)?,
        HTTPMethod::Post => post(preview_address, cookie, client, body)?,
    };

    if livereload {
        let ws_port: u16 = 8025;

        open_browser(&format!(
            "https://cloudflareworkers.com/ui/staging/index.html?wrangler_session_id={}\\&wrangler_ws_port={}\\&hide_editor#{}:{}{}",
            &session.to_string(), ws_port, script_id, https_str, preview_host,
        ))?;

        let server = WebSocket::new(|out| FiddleMessageServer { out })?
            .bind(format!("localhost:{}", ws_port))?;
        let broadcaster = server.broadcaster();
        thread::spawn(move || server.run());
        watch_for_changes(project, session.to_string(), broadcaster)?;
    } else {
        open_browser(&format!(
            "https://cloudflareworkers.com/#{}:{}{}",
            script_id, https_str, preview_host
        ))?;
        let msg = format!("Your worker responded with: {}", worker_res);
        message::preview(&msg);
    }

    Ok(())
}

fn open_browser(url: &str) -> Result<(), failure::Error> {
    let windows_cmd = format!("start {}", url);
    let mac_cmd = format!("open {}", url);
    let linux_cmd = format!("xdg-open {}", url);

    let _output = if cfg!(target_os = "windows") {
        Command::new("cmd").args(&["/C", &windows_cmd]).output()?
    } else if cfg!(target_os = "linux") {
        Command::new("sh").arg("-c").arg(&linux_cmd).output()?
    } else {
        Command::new("sh").arg("-c").arg(&mac_cmd).output()?
    };

    Ok(())
}

fn get(
    preview_address: &str,
    cookie: String,
    client: reqwest::Client,
) -> Result<String, failure::Error> {
    let res = client.get(preview_address).header("Cookie", cookie).send();
    let msg = format!("GET {}", preview_address);
    message::preview(&msg);
    Ok(res?.text()?)
}

fn post(
    preview_address: &str,
    cookie: String,
    client: reqwest::Client,
    body: Option<String>,
) -> Result<String, failure::Error> {
    let res = match body {
        Some(s) => client
            .post(preview_address)
            .header("Cookie", cookie)
            .body(s)
            .send(),
        None => client.post(preview_address).header("Cookie", cookie).send(),
    };
    let msg = format!("POST {}", preview_address);
    message::preview(&msg);
    Ok(res?.text()?)
}

fn watch_for_changes(
    project: &Project,
    session_id: String,
    broadcaster: Sender,
) -> Result<(), failure::Error> {
    let (tx, rx) = channel();
    watch_and_build(project, Some(tx))?;

    while let Ok(_e) = rx.recv() {
        if let Ok(new_id) = upload_and_get_id(project) {
            let msg = FiddleMessage {
                session_id: session_id.clone(),
                data: FiddleMessageData::LiveReload { new_id },
            };

            match broadcaster.send(serde_json::to_string(&msg)?) {
                Ok(_) => {
                    message::preview("Sent new id to preview!");
                }
                Err(_e) => message::user_error("communication with preview failed"),
            }
        }
    }

    broadcaster.shutdown()?;

    Ok(())
}

#[derive(Debug, Deserialize)]
struct Preview {
    id: String,
}

fn upload_and_get_id(project: &Project) -> Result<String, failure::Error> {
    let create_address = "https://cloudflareworkers.com/script";
    let client = http::client();

    let res = client
        .post(create_address)
        .multipart(build_script_upload_form(project)?)
        .send();

    let p: Preview = serde_json::from_str(&res?.text()?)?;

    Ok(p.id)
}
