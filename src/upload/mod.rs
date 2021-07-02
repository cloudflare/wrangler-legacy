pub mod form;
mod krate;
pub mod package;

use indicatif::{ProgressBar, ProgressStyle};
pub use package::Package;

use anyhow::Result;
use reqwest::blocking::Client;

use crate::settings::toml::Target;
use crate::sites::AssetManifest;

pub fn script(
    client: &Client,
    target: &Target,
    asset_manifest: Option<AssetManifest>,
) -> Result<()> {
    let worker_addr = format!(
        "https://api.cloudflare.com/client/v4/accounts/{}/workers/scripts/{}",
        target.account_id.load()?,
        target.name,
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

    if !res.status().is_success() {
        anyhow::bail!(error_msg(res.text()?))
    }

    Ok(())
}

fn error_msg(text: String) -> String {
    if text.contains("\"code\": 10034,") {
        "You need to verify your account's email address before you can publish. You can do this by checking your email or logging in to https://dash.cloudflare.com.".into()
    } else if text.contains("\"code\": 10000,") {
        "Your user configuration is invalid, please run wrangler login or wrangler config and enter a new set of credentials.".into()
    } else if text.contains("\"code\": 10075,") {
        "Setting a Usage Model requires a Paid plan with Unbound enabled. You can do this in the dash by logging in to https://dash.cloudflare.com/?account=workers/plans".into()
    } else {
        crate::format_api_errors(text)
    }
}

#[test]
fn fails_with_good_error_msg_on_verify_email_err() {
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
    let result = error_msg(text);
    assert!(result.contains("https://dash.cloudflare.com"));
}
