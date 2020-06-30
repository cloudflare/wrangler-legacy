use crate::settings::global_user::GlobalUser;
use crate::settings::toml::{DeployConfig, Target};
use crate::sites::{self, AssetManifest};
use crate::upload;

use reqwest::Url;
use serde::{Deserialize, Serialize};
use serde_json::json;

pub(super) fn upload(
    target: &mut Target,
    asset_manifest: Option<AssetManifest>,
    deploy_config: &DeployConfig,
    user: &GlobalUser,
    preview_token: String,
) -> Result<String, failure::Error> {
    let client = crate::http::legacy_auth_client(&user);
    if target.site.is_some() {
        sites::add_namespace(user, target, true)?;
    }

    let session_config = get_session_config(deploy_config);
    let address = get_upload_address(target);

    let script_upload_form = upload::form::build(target, asset_manifest, Some(session_config))?;

    let response = client
        .post(&address)
        .header("cf-preview-upload-config-token", preview_token)
        .multipart(script_upload_form)
        .send()?
        .error_for_status()?;

    let text = &response.text()?;

    // TODO: use cloudflare-rs for this :)
    let response: PreviewV4ApiResponse = serde_json::from_str(text)?;
    Ok(response.result.preview_token)
}

#[derive(Debug, Clone)]
pub struct Init {
    pub host: String,
    pub websocket_url: Url,
    pub preview_token: String,
}

impl Init {
    pub fn new(
        target: &Target,
        deploy_config: &DeployConfig,
        user: &GlobalUser,
    ) -> Result<Init, failure::Error> {
        let exchange_url = get_exchange_url(deploy_config, user)?;
        let host = match exchange_url.host_str() {
            Some(host) => Ok(host.to_string()),
            None => Err(failure::format_err!(
                "Could not parse host from exchange url"
            )),
        }?;

        let host = match deploy_config {
            DeployConfig::Zoned(_) => host,
            DeployConfig::Zoneless(_) => {
                let namespaces: Vec<&str> = host.as_str().split('.').collect();
                let subdomain = namespaces[1];
                format!("{}.{}.workers.dev", target.name, subdomain)
            }
        };

        let client = crate::http::legacy_auth_client(&user);
        let response = client.get(exchange_url).send()?.error_for_status()?;
        let text = &response.text()?;
        let response: InspectorV4ApiResponse = serde_json::from_str(text)?;
        let full_url = format!(
            "{}?{}={}",
            &response.inspector_websocket, "cf_workers_preview_token", &response.token
        );
        let websocket_url = Url::parse(&full_url)?;
        let preview_token = response.token;

        Ok(Init {
            host,
            websocket_url,
            preview_token,
        })
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
            "https://api.cloudflare.com/client/v4/zones/{}/workers/edge-preview",
            config.zone_id
        ),
        // TODO: zoneless is probably wrong
        DeployConfig::Zoneless(config) => format!(
            "https://api.cloudflare.com/client/v4/accounts/{}/workers/subdomain/edge-preview",
            config.account_id
        ),
    }
}

fn get_upload_address(target: &mut Target) -> String {
    format!(
        "https://api.cloudflare.com/client/v4/accounts/{}/workers/scripts/{}/edge-preview",
        target.account_id, target.name
    )
}

fn get_exchange_url(
    deploy_config: &DeployConfig,
    user: &GlobalUser,
) -> Result<Url, failure::Error> {
    let client = crate::http::legacy_auth_client(&user);
    let address = get_initialize_address(deploy_config);
    let url = Url::parse(&address)?;
    let response = client.get(url).send()?.error_for_status()?;
    let text = &response.text()?;
    let response: InitV4ApiResponse = serde_json::from_str(text)?;
    let url = Url::parse(&response.result.exchange_url)?;
    Ok(url)
}

#[derive(Debug, Serialize, Deserialize)]
struct InitResponse {
    pub exchange_url: String,
    pub token: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct InitV4ApiResponse {
    pub result: InitResponse,
}

#[derive(Debug, Serialize, Deserialize)]
struct InspectorV4ApiResponse {
    pub inspector_websocket: String,
    pub token: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct Preview {
    pub preview_token: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct PreviewV4ApiResponse {
    pub result: Preview,
}
