pub mod form;
mod krate;
pub mod package;

use indicatif::{ProgressBar, ProgressStyle};
pub use package::Package;

use anyhow::Result;
use reqwest::blocking::Client;

use crate::settings::toml::Target;
use crate::sites::AssetManifest;
use crate::TEMP_NOTICE_ES_MODULES_DO_BETA;

pub fn script(
    client: &Client,
    target: &Target,
    asset_manifest: Option<AssetManifest>,
) -> Result<()> {
    let worker_addr = format!(
        "https://api.cloudflare.com/client/v4/accounts/{}/workers/scripts/{}",
        target.account_id, target.name,
    );

    let script_upload_form = form::build(target, asset_manifest, None)?;

    let style = ProgressStyle::default_spinner().template("{spinner}   {msg}");
    let spinner = ProgressBar::new_spinner().with_style(style);
    spinner.set_message("Uploading script...");
    spinner.enable_steady_tick(20);

    let res = client
        .put(&worker_addr)
        .multipart(script_upload_form)
        .send()?;

    spinner.finish_and_clear();

    let res_status = res.status();

    if !res_status.is_success() {
        let res_text = res.text()?;
        anyhow::bail!(error_msg(res_status, res_text))
    }

    if let Some(usage_model) = target.usage_model {
        let addr = format!(
            "https://api.cloudflare.com/client/v4/accounts/{}/workers/scripts/{}/usage-model",
            target.account_id, target.name,
        );

        let res = client
            .put(&addr)
            .json(&serde_json::json!({
                "usage_model": usage_model.as_ref()
            }))
            .send()?;

        let res_status = res.status();

        if !res_status.is_success() {
            let res_text = res.text()?;
            anyhow::bail!(error_msg(res_status, res_text))
        }
    }

    Ok(())
}

fn error_msg(status: reqwest::StatusCode, text: String) -> String {
    if text.contains("\"code\": 10034,") {
        "You need to verify your account's email address before you can publish. You can do this by checking your email or logging in to https://dash.cloudflare.com.".to_string()
    } else if text.contains("\"code\": 10000,") {
        "Your user configuration is invalid, please run wrangler login or wrangler config and enter a new set of credentials.".to_string()
    } else if text.contains("\"code\": 10075,") {
        "Setting a Usage Model requires a Paid plan with Unbound enabled. You can do this in the dash by logging in to https://dash.cloudflare.com/?account=workers/plans".to_string()
    } else if text.contains("\"code\": 10015,") || text.contains("workers.api.error.not_entitled") {
        TEMP_NOTICE_ES_MODULES_DO_BETA.into()
    } else {
        format!("Something went wrong! Status: {}, Details {}", status, text)
    }
}

#[test]
fn fails_with_good_error_msg_on_verify_email_err() {
    let status = reqwest::StatusCode::FORBIDDEN;
    let text = r#"{
  "result": null,
  "success": false,
  "errors": [
    {
      "code": 10034,
      "message": "workers.api.error.email_verification_required"
    }
  ],
  "messages": []
}"#
    .to_string();
    let result = error_msg(status, text);
    assert!(result.contains("https://dash.cloudflare.com"));
}
