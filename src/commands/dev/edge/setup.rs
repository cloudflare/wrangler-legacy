use std::path::Path;

use crate::deploy::DeployTarget;
use crate::kv::bulk;
use crate::settings::global_user::GlobalUser;
use crate::settings::toml::Target;
use crate::sites::{add_namespace, sync};
use crate::terminal::message::{Message, StdOut};
use crate::upload;

use anyhow::{anyhow, Result};
use reqwest::Url;
use serde::{Deserialize, Serialize};
use serde_json::json;

pub(super) fn upload(
    target: &mut Target,
    deploy_target: &DeployTarget,
    user: &GlobalUser,
    session_token: String,
    verbose: bool,
) -> Result<String> {
    let client = crate::http::legacy_auth_client(&user);

    let (to_delete, asset_manifest, site_namespace_id) = if let Some(site_config) =
        target.site.clone()
    {
        let site_namespace = add_namespace(user, target, true)?;
        let path = Path::new(&site_config.bucket);
        let (to_upload, to_delete, asset_manifest) = sync(target, user, &site_namespace.id, path)?;

        // First, upload all existing files in given directory
        if verbose {
            StdOut::info("Uploading updated files...");
        }

        bulk::put(target, user, &site_namespace.id, to_upload, &None)?;
        (to_delete, Some(asset_manifest), Some(site_namespace.id))
    } else {
        (Vec::new(), None, None)
    };

    let session_config = get_session_config(deploy_target);
    let address = get_upload_address(target);

    let script_upload_form = upload::form::build(target, asset_manifest, Some(session_config))?;

    let response = client
        .post(&address)
        .header("cf-preview-upload-config-token", session_token)
        .multipart(script_upload_form)
        .send()?
        .error_for_status()?;

    if !to_delete.is_empty() {
        if verbose {
            StdOut::info("Deleting stale files...");
        }

        bulk::delete(target, user, &site_namespace_id.unwrap(), to_delete, &None)?;
    }

    let text = &response.text()?;

    // TODO: use cloudflare-rs for this :)
    let response: PreviewV4ApiResponse = serde_json::from_str(text)?;
    Ok(response.result.preview_token)
}

#[derive(Debug, Clone)]
pub struct Session {
    pub host: String,
    pub websocket_url: Url,
    pub preview_token: String,
}

impl Session {
    pub fn new(
        target: &Target,
        user: &GlobalUser,
        deploy_target: &DeployTarget,
    ) -> Result<Session> {
        let exchange_url = get_exchange_url(deploy_target, user)?;
        let host = match exchange_url.host_str() {
            Some(host) => Ok(host.to_string()),
            None => Err(anyhow!("Could not parse host from exchange url")),
        }?;

        let host = match deploy_target {
            DeployTarget::Zoned(_) => host,
            DeployTarget::Zoneless(_) => {
                let namespaces: Vec<&str> = host.as_str().split('.').collect();
                let subdomain = namespaces[1];
                format!("{}.{}.workers.dev", target.name, subdomain)
            }
            _ => unreachable!(),
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

        Ok(Session {
            host,
            websocket_url,
            preview_token,
        })
    }
}

fn get_session_config(target: &DeployTarget) -> serde_json::Value {
    match target {
        DeployTarget::Zoned(config) => {
            let mut routes: Vec<String> = Vec::new();
            for route in &config.routes {
                routes.push(route.pattern.clone());
            }
            json!({ "routes": routes })
        }
        DeployTarget::Zoneless(_) => json!({"workers_dev": true}),
        _ => unreachable!(),
    }
}

fn get_session_address(target: &DeployTarget) -> String {
    match target {
        DeployTarget::Zoned(config) => format!(
            "https://api.cloudflare.com/client/v4/zones/{}/workers/edge-preview",
            config.zone_id
        ),
        // TODO: zoneless is probably wrong
        DeployTarget::Zoneless(config) => format!(
            "https://api.cloudflare.com/client/v4/accounts/{}/workers/subdomain/edge-preview",
            config.account_id
        ),
        _ => unreachable!(),
    }
}

fn get_upload_address(target: &mut Target) -> String {
    format!(
        "https://api.cloudflare.com/client/v4/accounts/{}/workers/scripts/{}/edge-preview",
        target.account_id, target.name
    )
}

fn get_exchange_url(deploy_target: &DeployTarget, user: &GlobalUser) -> Result<Url> {
    let client = crate::http::legacy_auth_client(&user);
    let address = get_session_address(deploy_target);
    let url = Url::parse(&address)?;
    let response = client.get(url).send()?.error_for_status()?;
    let text = &response.text()?;
    let response: SessionV4ApiResponse = serde_json::from_str(text)?;
    let url = Url::parse(&response.result.exchange_url)?;
    Ok(url)
}

#[derive(Debug, Serialize, Deserialize)]
struct SessionResponse {
    pub exchange_url: String,
    pub token: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct SessionV4ApiResponse {
    pub result: SessionResponse,
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
