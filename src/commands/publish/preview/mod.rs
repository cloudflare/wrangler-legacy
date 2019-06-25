use std::process::Command;

mod http_method;
pub use http_method::HTTPMethod;

use crate::commands::publish;

use serde::Deserialize;
use uuid::Uuid;

use crate::http;
use crate::settings::project::{get_project_config, ProjectType};
use crate::terminal::message;

#[derive(Debug, Deserialize)]
struct Preview {
    pub id: String,
}

pub fn preview(
    method: Result<HTTPMethod, failure::Error>,
    body: Option<String>,
) -> Result<(), failure::Error> {
    let create_address = "https://cloudflareworkers.com/script";

    let client = http::client();

    let project_type = get_project_config()?.project_type;

    let res = match project_type {
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

    let session = Uuid::new_v4().to_simple();

    let preview_host = "example.com";
    let https = 1;
    let script_id = &p.id;

    let preview_address = "https://00000000000000000000000000000000.cloudflareworkers.com";
    let cookie = format!(
        "__ew_fiddle_preview={}{}{}{}",
        script_id, session, https, preview_host
    );

    let method = method.unwrap_or_default();

    let worker_res = match method {
        HTTPMethod::Get => get(preview_address, cookie, client)?,
        HTTPMethod::Post => post(preview_address, cookie, client, body)?,
    };
    let msg = format!("Your worker responded with: {}", worker_res);
    message::preview(&msg);

    open(preview_host, https, script_id)?;

    Ok(())
}

fn open(preview_host: &str, https: u8, script_id: &str) -> Result<(), failure::Error> {
    let https_str = match https {
        1 => "https://",
        0 => "http://",
        // hrm.
        _ => "",
    };

    let browser_preview = format!(
        "https://cloudflareworkers.com/#{}:{}{}",
        script_id, https_str, preview_host
    );
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
