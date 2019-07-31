use std::process::Command;

mod http_method;
pub use http_method::HTTPMethod;

use crate::commands::build;
use crate::commands::build_and_watch;
use crate::commands::publish;

use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::commands;
use crate::http;
<<<<<<< HEAD
use crate::settings::project::Project;
=======
use crate::install;
use crate::settings::project::get_project_config;
use crate::settings::project::{Project, ProjectType};
>>>>>>> c35288a... add build_and_watch
use crate::terminal::message;

use std::sync::mpsc::channel;
use std::thread;
use ws::WebSocket;

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
    let script_id = &upload_and_get_id()?;

    let preview_address = "https://00000000000000000000000000000000.cloudflareworkers.com";
    let cookie = format!(
        "__ew_fiddle_preview={}{}{}{}",
        script_id, session, https, preview_host
    );

    let method = method.unwrap_or_default();

    let client = http::client();
    let worker_res = match method {
        HTTPMethod::Get => get(preview_address, cookie, client)?,
        HTTPMethod::Post => post(preview_address, cookie, client, body)?,
    };

    let msg = format!("Your worker responded with: {}", worker_res);
    message::preview(&msg);

    open(preview_host, https, script_id, &session.to_string())?;

    if livereload {
        watch_for_changes(session.to_string())?;
    } else {
        println!("👷‍♀️ Your worker responded with: {}", worker_res);
    }

    Ok(())
}

fn open(
    preview_host: &str,
    https: bool,
    script_id: &str,
    session_id: &str,
) -> Result<(), failure::Error> {
    let https_str = if https { "https://" } else { "http://" };

    let browser_preview = if install::target::DEBUG {
        format!(
            "http://localhost:3000/src/test/manual/#{}:{}{}?session_id={}&noeditor=true",
            script_id, https_str, preview_host, session_id
        )
    } else {
        format!(
            "https://cloudflareworkers.com/#{}:{}{}?session_id={}&noeditor=true",
            script_id, https_str, preview_host, session_id
        )
    };
    let windows_cmd = format!("start {}", browser_preview);
    let mac_cmd = format!("open {}", browser_preview);
    let linux_cmd = format!("xdg-open {}", browser_preview);

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

//for now, this is only used by livereloading.
//in the future we may use this websocket for other things
//so support other message types
#[derive(Debug, Serialize)]
enum FiddleMessage {
    LiveReload { old_id: String, new_id: String },
}

fn watch_for_changes(original_id: String) -> Result<(), failure::Error> {
    //start up the websocket server.
    //needs a bs handler factory closure, even though we never respond
    let server = WebSocket::new(|_out| |_msg| Ok(()))?.bind("localhost:8025")?;
    let broadcaster = server.broadcaster();
    thread::spawn(move || server.run());

    let mut old_id = original_id;
    let (tx, rx) = channel();
    build_and_watch(&get_project_config()?, Some(tx));

    while let Ok(_e) = rx.recv() {
        if let Ok(new_id) = upload_and_get_id() {
            let msg = FiddleMessage::LiveReload {
                old_id: old_id.clone(),
                new_id: new_id.clone(),
            };

            match broadcaster.send(serde_json::to_string(&msg)?) {
                Ok(_) => {
                    println!("Sent new id to preview!");
                    old_id = new_id;
                }
                Err(_e) => println!("communication with preview failed"),
            }
        }
    }

    broadcaster.shutdown()?;

    Ok(())
}

#[derive(Debug, Deserialize)]
struct Preview {
    pub id: String,
}

fn upload_and_get_id() -> Result<String, failure::Error> {
    let create_address = "https://cloudflareworkers.com/script";
    let client = http::client();

    let res = match get_project_config()?.project_type {
        ProjectType::Rust => client
            .post(create_address)
            .multipart(publish::build_multipart_script()?)
            .send(),
        ProjectType::JavaScript => client
            .post(create_address)
            .body(publish::build_js_script()?)
            .send(),
        ProjectType::Webpack => client
            .post(create_address)
            .multipart(publish::build_webpack_form()?)
            .send(),
    };

    let p: Preview = serde_json::from_str(&res?.text()?)?;

    Ok(p.id)
}
