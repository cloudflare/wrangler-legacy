#[cfg(not(target_os = "windows"))]
use std::fs::File;
#[cfg(not(target_os = "windows"))]
use std::os::unix::fs::PermissionsExt;
#[cfg(not(target_os = "windows"))]
use std::path::Path;

use anyhow::Result;
use cloudflare::endpoints::user::{GetUserDetails, GetUserTokenStatus};
use cloudflare::framework::apiclient::ApiClient;

use crate::http;
use crate::settings::{get_global_config_path, global_user::GlobalUser};
use crate::terminal::message::{Message, StdOut};
use crate::terminal::styles;

// set the permissions on the dir, we want to avoid that other user reads to file
#[cfg(not(target_os = "windows"))]
pub fn set_file_mode(file: &Path) {
    File::open(&file)
        .unwrap()
        .set_permissions(PermissionsExt::from_mode(0o600))
        .expect("could not set permissions on file");
}

pub fn global_config(user: &GlobalUser, verify: bool) -> Result<()> {
    if verify {
        StdOut::info("Validating credentials...");
        validate_credentials(user)?;
    }

    let config_file = get_global_config_path();
    user.to_file(&config_file)?;

    // set permissions on the file
    #[cfg(not(target_os = "windows"))]
    set_file_mode(&config_file);

    StdOut::success(&format!(
        "Successfully configured. You can find your configuration file at: {}",
        &config_file.to_string_lossy()
    ));

    Ok(())
}

// validate_credentials() checks the /user/tokens/verify endpoint (for API token)
// or /user endpoint (for global API key) to ensure provided credentials actually work.
pub fn validate_credentials(user: &GlobalUser) -> Result<()> {
    let client = http::cf_v4_client(user)?;

    match user {
        GlobalUser::TokenAuth { .. } => match client.request(&GetUserTokenStatus {}) {
            Ok(success) => {
                if success.result.status == "active" {
                    Ok(())
                } else {
                    anyhow::bail!("Authentication check failed. Your token has status \"{}\", not \"active\".\nTry rolling your token on the Cloudflare dashboard.")
                }
            }
            Err(e) => anyhow::bail!(
                "Authentication check failed. Please make sure your API token is correct.\n{}",
                http::format_error(e, None)
            ),
        },
        GlobalUser::GlobalKeyAuth { .. } => match client.request(&GetUserDetails {}) {
            Ok(_) => Ok(()),
            Err(_) => {
                let api_docs_url = styles::url(
                    "https://developers.cloudflare.com/workers/quickstart/#global-api-key",
                );
                anyhow::bail!("Authentication check failed. Please make sure your email and global API key pair are correct.\nSee {}", api_docs_url)
            }
        },
    }
}
