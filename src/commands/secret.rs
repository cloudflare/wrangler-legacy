use cloudflare::endpoints::workers::{CreateSecret, CreateSecretParams, DeleteSecret, ListSecrets};
use cloudflare::framework::apiclient::ApiClient;
use cloudflare::framework::response::ApiFailure;

use anyhow::Result;

use crate::http;
use crate::settings::global_user::GlobalUser;
use crate::settings::toml::Target;
use crate::terminal::message::{Message, StdOut};
use crate::terminal::{emoji, interactive};
use crate::upload;

fn format_error(e: ApiFailure) -> String {
    http::format_error(e, Some(&secret_errors))
}

fn validate_target(target: &Target) -> Result<()> {
    let mut missing_fields = Vec::new();

    if target.account_id.is_empty() {
        missing_fields.push("account_id")
    };

    if !missing_fields.is_empty() {
        anyhow::bail!(
            "{} Your configuration file is missing the following field(s): {:?}",
            emoji::WARN,
            missing_fields
        )
    } else {
        Ok(())
    }
}

// secret_errors() provides more detailed explanations of API error codes.
fn secret_errors(error_code: u16) -> &'static str {
    match error_code {
        7003 | 7000 => {
            "Your configuration file is likely missing the field \"account_id\", which is required to create a secret."
        }
        10053 => "There is already another binding with a different type by this name. Check your configuration file or your Cloudflare dashboard for conflicting bindings",
        10054 => "Your secret is too large, it must be 5kB or less",
        10055 => "You have exceeded the limit of 32 text bindings for this worker. Run `wrangler secret list` or go to your Cloudflare dashboard to clean up unused text/secret variables",
        _ => "",
    }
}

// upload_draft_worker will attempt to upload a "draft" version of a worker script if it does not
// already exist in the API (API error code 10007 is returned). The function returns None if this draft
// script was uploaded, or else returns Some (with a Result type so we can return a potential script upload error
// up the call chain).
pub fn upload_draft_worker(
    e: &ApiFailure,
    user: &GlobalUser,
    target: &Target,
) -> Option<Result<()>> {
    match e {
        ApiFailure::Error(_, api_errors) => {
            let error = &api_errors.errors[0];
            if error.code == 10007 {
                StdOut::working(&format!("Worker {} doesn't exist in the API yet. Creating a draft Worker so we can create new secret.", target.name));
                let upload_client = http::legacy_auth_client(user);
                Some(upload::script(&upload_client, target, None))
            } else {
                None
            }
        }
        ApiFailure::Invalid(_) => None,
    }
}

pub fn create_secret(name: &str, user: &GlobalUser, target: &Target) -> Result<()> {
    validate_target(target)?;

    let secret_value = interactive::get_user_input_multi_line(&format!(
        "Enter the secret text you'd like assigned to the variable {} on the script named {}:",
        name, target.name
    ));

    if secret_value.is_empty() {
        anyhow::bail!("Your secret cannot be empty.")
    }

    StdOut::working(&format!(
        "Creating the secret for script name {}",
        target.name
    ));

    let client = http::cf_v4_client(user)?;

    let params = CreateSecretParams {
        name: name.to_string(),
        text: secret_value,
        secret_type: "secret_text".to_string(),
    };

    let response = client.request(&CreateSecret {
        account_identifier: &target.account_id,
        script_name: &target.name,
        params: params.clone(),
    });

    match response {
        Ok(_) => StdOut::success(&format!("Success! Uploaded secret {}.", name)),
        Err(e) => match upload_draft_worker(&e, user, target) {
            None => anyhow::bail!(format_error(e)),
            Some(draft_upload_response) => match draft_upload_response {
                Ok(_) => {
                    let retry_response = client.request(&CreateSecret {
                        account_identifier: &target.account_id,
                        script_name: &target.name,
                        params,
                    });

                    match retry_response {
                        Ok(_) => StdOut::success(&format!("Success! Uploaded secret {}.", name)),
                        Err(e) => anyhow::bail!(format_error(e)),
                    }
                }
                Err(e) => anyhow::bail!(e),
            },
        },
    }

    Ok(())
}

pub fn delete_secret(name: &str, user: &GlobalUser, target: &Target) -> Result<()> {
    validate_target(target)?;

    match interactive::confirm(&format!(
        "Are you sure you want to permanently delete the variable {} on the script named {}?",
        name, target.name
    )) {
        Ok(true) => (),
        Ok(false) => {
            StdOut::info(&format!("Not deleting secret {}.", name));
            return Ok(());
        }
        Err(e) => anyhow::bail!(e),
    }

    StdOut::working(&format!(
        "Deleting the secret {} on script {}.",
        name, target.name
    ));

    let client = http::cf_v4_client(user)?;

    let response = client.request(&DeleteSecret {
        account_identifier: &target.account_id,
        script_name: &target.name,
        secret_name: &name,
    });

    match response {
        Ok(_) => StdOut::success(&format!("Success! Deleted secret {}.", name)),
        Err(e) => anyhow::bail!(format_error(e)),
    }

    Ok(())
}

pub fn list_secrets(user: &GlobalUser, target: &Target) -> Result<()> {
    validate_target(target)?;
    let client = http::cf_v4_client(user)?;

    let response = client.request(&ListSecrets {
        account_identifier: &target.account_id,
        script_name: &target.name,
    });

    match response {
        Ok(success) => {
            let secrets = success.result;
            println!("{}", serde_json::to_string(&secrets)?);
        }
        Err(e) => anyhow::bail!(format_error(e)),
    }

    Ok(())
}
