pub mod form;
mod krate;
pub mod package;

pub use package::Package;

use crate::commands::kv::bucket::AssetManifest;
use crate::http;
use crate::settings::global_user::GlobalUser;
use crate::settings::toml::Target;

pub fn script(
    user: &GlobalUser,
    target: &Target,
    asset_manifest: Option<AssetManifest>,
) -> Result<(), failure::Error> {
    let worker_addr = format!(
        "https://api.cloudflare.com/client/v4/accounts/{}/workers/scripts/{}",
        target.account_id, target.name,
    );

    let client = if target.site.is_some() {
        http::auth_client(Some("site"), user)
    } else {
        http::auth_client(None, user)
    };

    let script_upload_form = form::build(target, asset_manifest)?;

    let res = client
        .put(&worker_addr)
        .multipart(script_upload_form)
        .send()?;

    let res_status = res.status();

    if !res_status.is_success() {
        let res_text = res.text()?;
        failure::bail!(error_msg(res_status, res_text))
    }

    Ok(())
}

fn error_msg(status: reqwest::StatusCode, text: String) -> String {
    if text.contains("\"code\": 10034,") {
        "You need to verify your account's email address before you can publish. You can do this by checking your email or logging in to https://dash.cloudflare.com.".to_string()
    } else if text.contains("\"code\":10000,") {
        "Your user configuration is invalid, please run wrangler config and enter a new set of credentials.".to_string()
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
