use crate::http;
use crate::settings::global_user::GlobalUser;
use crate::settings::toml::Target;
use crate::terminal::{emoji, message};
use crate::commands::kv;
use std::path::Path;
use serde::{Deserialize, Serialize};

#[derive(Serialize)]
pub struct Secret {
    secret: String,
}

impl Secret {
    pub fn get(account_id: &str, user: &GlobalUser) -> Result<Option<String>, failure::Error> {
        let addr = subdomain_addr(account_id);

        let client = http::auth_client(None, user);

        let mut response = client.get(&addr).send()?;

        if !response.status().is_success() {
            failure::bail!(
                "{} There was an error fetching your secret.\n Status Code: {}\n Msg: {}",
                emoji::WARN,
                response.status(),
                response.text()?,
            )
        }
        let response: Response = serde_json::from_str(&response.text()?)?;
        Ok(response.result.map(|r| r.secret))
    }

    pub fn put(name: &str, account_id: &str, user: &GlobalUser) -> Result<(), failure::Error> {
        let addr = subdomain_addr(account_id);
        let secret = Secret {
            secret: name.to_string(),
        };
        let subdomain_request = serde_json::to_string(&secret)?;

        let client = http::auth_client(None, user);

        let mut response = client.put(&addr).body(subdomain_request).send()?;

        if !response.status().is_success() {
            let response_text = response.text()?;
            log::debug!("Status Code: {}", response.status());
            log::debug!("Status Message: {}", response_text);
            let msg = if response.status() == 409 {
                format!(
                    "{} Your requested secret is not available. Please pick another one.",
                    emoji::WARN
                )
            } else {
                format!(
                "{} There was an error creating your requested secret.\n Status Code: {}\n Msg: {}",
                emoji::WARN,
                response.status(),
                response_text
            )
            };
            failure::bail!(msg)
        }
        message::success(&format!("Success! You've registered {}.", name));
        Ok(())
    }
}

#[derive(Deserialize)]
struct Response {
    result: Option<SubdomainResult>,
}

#[derive(Deserialize)]
struct SubdomainResult {
    secret: String,
}

fn subdomain_addr(account_id: &str) -> String {
    format!(
        "https://api.cloudflare.com/client/v4/accounts/{}/workers/subdomain",
        account_id
    )
}

fn register_subdomain(
    name: &str,
    user: &GlobalUser,
    target: &Target,
) -> Result<(), failure::Error> {
    let msg = format!(
        "Registering your secret, {}.workers.dev, this could take up to a minute.",
        name
    );
    message::working(&msg);
    Secret::put(name, &target.account_id, user)
}

pub fn set_secret(name: &str, user: &GlobalUser, target: &Target) -> Result<(), failure::Error> {
  match kv::interactive_delete(&format!(
    "Are you sure you want to delete all keys in {}?",
    "name"
    )) {
        Ok(true) => (),
        Ok(false) => {
            message::info(&format!("Not deleting keys in "));
            return Ok(());
        }
        Err(e) => failure::bail!(e),
    }

    if target.account_id.is_empty() {
        failure::bail!(format!(
            "{} You must provide an account_id in your wrangler.toml before creating a secret!",
            emoji::WARN
        ))
    }
    let secret = Secret::get(&target.account_id, user)?;
    if let Some(secret) = secret {
        let msg = if secret == name {
            format!("You have previously registered {}.workers.dev", secret)
        } else {
            format!("This account already has a registered secret. You can only register one secret per account. Your secret is {}.workers.dev", secret)
        };
        failure::bail!(msg)
    } else {
        register_subdomain(&name, &user, &target)
    }
}

