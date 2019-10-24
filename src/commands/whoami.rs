use crate::settings::global_user::GlobalUser;
use crate::terminal::{emoji, message};

use cloudflare::endpoints::user::GetUserDetails;
use cloudflare::framework::apiclient::ApiClient;
use cloudflare::framework::response::ApiFailure;
use cloudflare::framework::auth::Credentials;
use cloudflare::framework::{Environment, HttpApiClient, HttpApiClientConfig};

pub fn whoami(user: &GlobalUser) -> Result<(), failure::Error> {
    // If using email + API key for auth, simply prints out email from config file.
    let email: String = match &user.email {
        Some(email) => email.to_string(),
        None => {
            // If no email found in config (implies API token usage) fall back to getting it
            // via API using API token.
            match &user.api_token {
                Some(_) => get_email_from_api(&user)?,
                None => failure::bail!("No email + API key pair or API token found in config, cannot get information about user"),
            }
        }
    };

    let msg = format!(
        "{} You are logged with the email '{}'.",
        emoji::WAVING,
        email
    );
    message::info(&msg);
    Ok(())
}

// If only an API token is present in the user's config, we can still get their email
// via the API.
fn get_email_from_api(user: &GlobalUser) -> Result<String, failure::Error> {
    let client = HttpApiClient::new(
        Credentials::from(user.to_owned()),
        HttpApiClientConfig::default(),
        Environment::Production,
    )?;

    let result = client.request(&GetUserDetails {});
    match result {
        Ok(success) => Ok(success.result.email),
        Err(e) => {
            match e {
                ApiFailure::Error(status, errors) => {
                    let mut err_msg = "".to_string();
                    err_msg.push_str(&format!("HTTP {}", status));
                    for err in errors.errors {
                        err_msg.push_str(&format!("\nError {}: {}", err.code, err.message));
                        match err.code {
                            9109 => err_msg.push_str(". If you are using an API token, it may not have permission to access user details"),
                            _ => (),
                        }
                    }
                    failure::bail!("{}", err_msg);
                }
                ApiFailure::Invalid(reqwest_err) => failure::bail!("Error: {}", reqwest_err),
            }
        }
    }
}
