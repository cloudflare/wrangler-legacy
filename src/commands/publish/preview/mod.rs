use std::process::Command;

mod http_method;
pub use http_method::HTTPMethod;

use crate::commands::publish;

use serde::Deserialize;
use uuid::Uuid;

#[derive(Debug, Deserialize)]
struct Preview {
    pub id: String,
}

pub fn preview(
    method: Result<HTTPMethod, failure::Error>,
    body: Option<String>,
) -> Result<(), failure::Error> {
    let create_address = "https://cloudflareworkers.com/script";

    let client = reqwest::Client::new();
    let res = client
        .post(create_address)
        .multipart(publish::build_form()?)
        .send();

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
    println!("{}", &p.id);

    let method = method.unwrap_or_default();

    let worker_res = match method {
        HTTPMethod::Get => get(preview_address, cookie, client)?,
        HTTPMethod::Post => post(preview_address, cookie, client, body)?,
    };
    println!("{}", worker_res);

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
    Ok(res?.text()?)
}
