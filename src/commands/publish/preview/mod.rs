use std::process::Command;

mod http_method;
pub use http_method::HTTPMethod;

use crate::commands::publish;

use serde::Deserialize;
use uuid::Uuid;

use crate::commands;
use crate::http;
use crate::settings::global_user::GlobalUser;
use crate::settings::project::Project;
use crate::terminal::message;

// Using this instead of just `https://cloudflareworkers.com` returns just the worker response to the CLI
const PREVIEW_ADDRESS: &str = "https://00000000000000000000000000000000.cloudflareworkers.com";

#[derive(Debug, Deserialize)]
struct Preview {
    pub id: String,
}

impl From<ApiPreview> for Preview {
    fn from(api_preview: ApiPreview) -> Preview {
        Preview {
            id: api_preview.preview_id,
        }
    }
}

#[derive(Debug, Deserialize)]
struct ApiPreview {
    pub preview_id: String,
}

#[derive(Debug, Deserialize)]
struct V4ApiResponse {
    pub result: ApiPreview,
}

pub fn preview(
    project: &Project,
    user: Option<GlobalUser>,
    method: HTTPMethod,
    body: Option<String>,
) -> Result<(), failure::Error> {
    commands::build(&project)?;

    let client = match &user {
        Some(user) => http::auth_client(&user),
        None => http::client(),
    };

    let preview = upload_to_preview(&project, user, &client)?;

    let session = Uuid::new_v4().to_simple();

    let preview_host = "example.com";
    let https = 1;
    let script_id = &preview.id;

    let cookie = format!(
        "__ew_fiddle_preview={}{}{}{}",
        script_id, session, https, preview_host
    );

    let worker_res = match method {
        HTTPMethod::Get => get(cookie, &client)?,
        HTTPMethod::Post => post(cookie, &client, body)?,
    };
    let msg = format!("Your worker responded with: {}", worker_res);
    message::preview(&msg);

    open(preview_host, https, script_id)?;

    Ok(())
}

fn upload_to_preview(
    project: &Project,
    user: Option<GlobalUser>,
    client: &reqwest::Client,
) -> Result<Preview, failure::Error> {
    let create_address = match &user {
        Some(_user) => format!(
            "https://api.cloudflare.com/client/v4/accounts/{}/workers/scripts/{}/preview",
            project.account_id, project.name
        ),
        None => "https://cloudflareworkers.com/script".to_string(),
    };
    log::info!("address: {}", create_address);

    let script_upload_form = publish::build_script_upload_form(project)?;

    let mut res = client
        .post(&create_address)
        .multipart(script_upload_form)
        .send()?
        .error_for_status()?;

    let text = &res.text()?;
    log::info!("Response from preview: {:#?}", text);

    match user {
        Some(_user) => {
            let response: V4ApiResponse = serde_json::from_str(text)
                .expect("could not create a script on cloudflareworkers.com");

            Ok(Preview::from(response.result))
        }
        None => {
            Ok(serde_json::from_str(text)
                .expect("could not create a script on cloudflareworkers.com"))
        }
    }
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

fn get(cookie: String, client: &reqwest::Client) -> Result<String, failure::Error> {
    let res = client.get(PREVIEW_ADDRESS).header("Cookie", cookie).send();
    let msg = format!("GET {}", PREVIEW_ADDRESS);
    message::preview(&msg);
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
