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
use reqwest::Client;

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

// When making authenticated preview requests, we go through the v4 Workers API rather than
// hitting the preview service directly, so its response is formatted like a v4 API response.
// These structs are here to convert from this format into the Preview defined above.
#[derive(Debug, Deserialize)]
struct ApiPreview {
    pub preview_id: String,
}

#[derive(Debug, Deserialize)]
struct V4ApiResponse {
    pub result: ApiPreview,
}

pub fn preview(
    mut project: Project,
    user: Option<GlobalUser>,
    method: HTTPMethod,
    body: Option<String>,
) -> Result<(), failure::Error> {
    let client: Client;

    let preview = match &user {
        Some(user) => {
            log::info!("GlobalUser set, running with authentication");

            super::validate_project(&project, false)?;

            commands::build(&project)?;
            client = http::auth_client(&user);

            authenticated_upload(&client, &project)?
        }
        None => {
            log::info!("GlobalUser not set, running without authentication");

            // KV namespaces are not supported by the preview service unless you authenticate
            // so we omit them and provide the user with a little guidance. We don't error out, though,
            // because there are valid workarounds for this for testing purposes.
            if project.kv_namespaces.is_some() {
                message::warn("KV Namespaces are not supported without setting API credentials");
                message::help(
                    "Run `wrangler config` or set $CF_API_KEY and $CF_EMAIL to configure your user.",
                );
                project.kv_namespaces = None;
            }

            commands::build(&project)?;
            client = http::client();

            unauthenticated_upload(&client, &project)?
        }
    };

    let worker_res = call_worker(&client, preview, method, body)?;

    let msg = format!("Your worker responded with: {}", worker_res);
    message::preview(&msg);

    Ok(())
}

fn authenticated_upload(client: &Client, project: &Project) -> Result<Preview, failure::Error> {
    let create_address = format!(
        "https://api.cloudflare.com/client/v4/accounts/{}/workers/scripts/{}/preview",
        project.account_id, project.name
    );
    log::info!("address: {}", create_address);

    let script_upload_form = publish::build_script_upload_form(&project)?;

    let mut res = client
        .post(&create_address)
        .multipart(script_upload_form)
        .send()?
        .error_for_status()?;

    let text = &res.text()?;
    log::info!("Response from preview: {:#?}", text);

    let response: V4ApiResponse =
        serde_json::from_str(text).expect("could not create a script on cloudflareworkers.com");

    Ok(Preview::from(response.result))
}

fn unauthenticated_upload(client: &Client, project: &Project) -> Result<Preview, failure::Error> {
    let create_address = "https://cloudflareworkers.com/script";
    log::info!("address: {}", create_address);

    let script_upload_form = publish::build_script_upload_form(project)?;

    let mut res = client
        .post(create_address)
        .multipart(script_upload_form)
        .send()?
        .error_for_status()?;

    let text = &res.text()?;
    log::info!("Response from preview: {:#?}", text);

    let preview: Preview =
        serde_json::from_str(text).expect("could not create a script on cloudflareworkers.com");

    Ok(preview)
}

fn call_worker(
    client: &Client,
    preview: Preview,
    method: HTTPMethod,
    body: Option<String>,
) -> Result<String, failure::Error> {
    let session = Uuid::new_v4().to_simple();

    let preview_host = "example.com";
    let https = 1;
    let script_id = &preview.id;

    let cookie = format!(
        "__ew_fiddle_preview={}{}{}{}",
        script_id, session, https, preview_host
    );

    let res = match method {
        HTTPMethod::Get => get(cookie, &client)?,
        HTTPMethod::Post => post(cookie, &client, body)?,
    };

    open(preview_host, https, script_id)?;

    Ok(res)
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
