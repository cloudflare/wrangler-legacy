use crate::commands::kv::bucket::AssetManifest;
use crate::commands::publish;
use crate::http;
use crate::settings::global_user::GlobalUser;
use crate::settings::toml::{DeployConfig, Target};
use crate::upload;

use serde::{Deserialize, Serialize};
use serde_json::json;
use url::Url;

pub(super) fn upload(
    target: &mut Target,
    asset_manifest: Option<AssetManifest>,
    deploy_config: &DeployConfig,
    user: &GlobalUser,
    preview_token: String,
) -> Result<String, failure::Error> {
    let client = http::auth_client(None, &user);
    if let Some(site_config) = target.site.clone() {
        publish::bind_static_site_contents(user, target, &site_config, true)?;
    }

    let session_config = get_session_config(deploy_config);
    let address = get_upload_address(target);

    let script_upload_form = upload::form::build(target, asset_manifest, Some(session_config))?;

    let response = client
        .post(&address)
        .header("cf-workers-preview-token", preview_token)
        .multipart(script_upload_form)
        .send()?
        .error_for_status()?;

    let text = &response.text()?;

    // TODO: use cloudflare-rs for this :)
    let response: PreviewV4ApiResponse = serde_json::from_str(text)?;
    Ok(response.result.preview_token)
}

pub(super) fn get_preview_token(
    deploy_config: &DeployConfig,
    user: &GlobalUser,
) -> Result<String, failure::Error> {
    let exchange_url = get_exchange_url(deploy_config, user)?;
    let client = http::auth_client(None, &user);
    let response = client.get(&exchange_url).send()?.error_for_status()?;
    let headers = response.headers();
    let token = headers.get("cf-workers-preview-token");
    match token {
        Some(token) => Ok(token.to_str()?.to_string()),
        None => failure::bail!("Could not get token to initialize preview session"),
    }
}

fn get_session_config(deploy_config: &DeployConfig) -> serde_json::Value {
    match deploy_config {
        DeployConfig::Zoned(config) => {
            let mut routes: Vec<String> = Vec::new();
            for route in &config.routes {
                routes.push(route.pattern.clone());
            }
            json!({ "routes": routes })
        }
        DeployConfig::Zoneless(_) => json!({"workers_dev": true}),
    }
}

fn get_initialize_address(deploy_config: &DeployConfig) -> String {
    match deploy_config {
        DeployConfig::Zoned(config) => format!(
            "https://api.cloudflare.com/client/v4/zones/{}/workers/realish",
            config.zone_id
        ),
        // TODO: zoneless is probably wrong
        DeployConfig::Zoneless(config) => format!(
            "https://api.cloudflare.com/client/v4/account/{}/workers/realish/subdomain",
            config.account_id
        ),
    }
}

fn get_upload_address(target: &mut Target) -> String {
    format!(
        "https://api.cloudflare.com/client/v4/accounts/{}/workers/scripts/{}/realish",
        target.account_id, target.name
    )
}

fn get_exchange_url(
    deploy_config: &DeployConfig,
    user: &GlobalUser,
) -> Result<String, failure::Error> {
    let client = http::auth_client(None, &user);
    let address = get_initialize_address(deploy_config);
    let url = Url::parse(&address)?;
    let response = client.get(url).send()?.error_for_status()?;
    let text = &response.text()?;
    let response: InitV4ApiResponse = serde_json::from_str(text)?;
    Ok(response.result.exchange_url)
}

#[derive(Debug, Serialize, Deserialize)]
struct Init {
    pub exchange_url: String,
    pub token: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct InitV4ApiResponse {
    pub result: Init,
}

#[derive(Debug, Serialize, Deserialize)]
struct Preview {
    pub preview_token: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct PreviewV4ApiResponse {
    pub result: Preview,
}
