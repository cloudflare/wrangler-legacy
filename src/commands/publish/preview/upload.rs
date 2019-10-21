use crate::commands::kv::bucket::AssetManifest;
use crate::commands::publish;
use crate::http;
use crate::settings::global_user::GlobalUser;
use crate::settings::target::Target;
use crate::terminal::message;
use reqwest::Client;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
struct Preview {
    id: String,
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

const SITES_UNAUTH_PREVIEW_ERR: &str =
    "Unauthenticated preview does not work for previewing Workers Sites; you need to \
     authenticate to upload your site contents.";

// Builds and uploads the script and its bindings. Returns the ID of the uploaded script.
pub fn build_and_upload(
    target: &mut Target,
    user: Option<&GlobalUser>,
    sites_preview: bool,
) -> Result<String, failure::Error> {
    let preview = match &user {
        Some(user) => {
            log::info!("GlobalUser set, running with authentication");

            let missing_fields = validate(&target);

            if missing_fields.is_empty() {
                let client = http::auth_client(None, &user);

                if let Some(site_config) = target.site.clone() {
                    publish::bind_static_site_contents(user, target, &site_config, true)?;
                }

                let asset_manifest = publish::upload_buckets(target, user)?;
                authenticated_upload(&client, &target, asset_manifest)?
            } else {
                message::warn(&format!(
                    "Your wrangler.toml is missing the following fields: {:?}",
                    missing_fields
                ));
                message::warn("Falling back to unauthenticated preview.");
                if sites_preview {
                    failure::bail!(SITES_UNAUTH_PREVIEW_ERR)
                }

                let client = http::client(None);
                unauthenticated_upload(&client, &target)?
            }
        }
        None => {
            message::warn(
                "You haven't run `wrangler config`. Running preview without authentication",
            );
            message::help(
                "Run `wrangler config` or set $CF_API_KEY and $CF_EMAIL to configure your user.",
            );

            if sites_preview {
                failure::bail!(SITES_UNAUTH_PREVIEW_ERR)
            }

            let client = http::client(None);

            unauthenticated_upload(&client, &target)?
        }
    };

    Ok(preview.id)
}

fn validate(target: &Target) -> Vec<&str> {
    let mut missing_fields = Vec::new();

    if target.account_id.is_empty() {
        missing_fields.push("account_id")
    };
    if target.name.is_empty() {
        missing_fields.push("name")
    };

    match &target.kv_namespaces {
        Some(kv_namespaces) => {
            for kv in kv_namespaces {
                if kv.binding.is_empty() {
                    missing_fields.push("kv-namespace binding")
                }

                if kv.id.is_empty() {
                    missing_fields.push("kv-namespace id")
                }
            }
        }
        None => {}
    }

    missing_fields
}

fn authenticated_upload(
    client: &Client,
    target: &Target,
    asset_manifest: Option<AssetManifest>,
) -> Result<Preview, failure::Error> {
    let create_address = format!(
        "https://api.cloudflare.com/client/v4/accounts/{}/workers/scripts/{}/preview",
        target.account_id, target.name
    );
    log::info!("address: {}", create_address);

    let script_upload_form = publish::build_script_and_upload_form(target, asset_manifest)?;

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

fn unauthenticated_upload(client: &Client, target: &Target) -> Result<Preview, failure::Error> {
    let create_address = "https://cloudflareworkers.com/script";
    log::info!("address: {}", create_address);

    // KV namespaces are not supported by the preview service unless you authenticate
    // so we omit them and provide the user with a little guidance. We don't error out, though,
    // because there are valid workarounds for this for testing purposes.
    let script_upload_form = if target.kv_namespaces.is_some() {
        message::warn(
            "KV Namespaces are not supported in preview without setting API credentials and account_id",
        );
        let mut target = target.clone();
        target.kv_namespaces = None;
        publish::build_script_and_upload_form(&target, None)?
    } else {
        publish::build_script_and_upload_form(&target, None)?
    };

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
