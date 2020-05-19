use crate::http;
use crate::settings::global_user::GlobalUser;
use crate::settings::toml::Target;
use crate::terminal::{emoji, message};

use serde::{Deserialize, Serialize};

#[derive(Serialize)]
pub struct Subdomain {
    subdomain: String,
}

impl Subdomain {
    pub fn get(account_id: &str, user: &GlobalUser) -> Result<Option<String>, failure::Error> {
        let addr = subdomain_addr(account_id);

        let client = http::legacy_auth_client(user);

        let response = client.get(&addr).send()?;

        if !response.status().is_success() {
            failure::bail!(
                "{} There was an error fetching your subdomain.\n Status Code: {}\n Msg: {}",
                emoji::WARN,
                response.status(),
                response.text()?,
            )
        }
        let response: Response = serde_json::from_str(&response.text()?)?;
        Ok(response.result.map(|r| r.subdomain))
    }

    pub fn put(name: &str, account_id: &str, user: &GlobalUser) -> Result<(), failure::Error> {
        let addr = subdomain_addr(account_id);
        let subdomain = Subdomain {
            subdomain: name.to_string(),
        };
        let subdomain_request = serde_json::to_string(&subdomain)?;

        let client = http::legacy_auth_client(user);

        let response = client.put(&addr).body(subdomain_request).send()?;

        let response_status = response.status();
        if !response_status.is_success() {
            let response_text = response.text()?;
            log::debug!("Status Code: {}", response_status);
            log::debug!("Status Message: {}", response_text);
            let msg = if response_status == 409 {
                format!(
                    "{} Your requested subdomain is not available. Please pick another one.",
                    emoji::WARN
                )
            } else {
                format!(
                "{} There was an error creating your requested subdomain.\n Status Code: {}\n Msg: {}",
                emoji::WARN,
                response_status,
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
    subdomain: String,
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
        "Registering your subdomain, {}.workers.dev, this could take up to a minute.",
        name
    );
    message::working(&msg);
    Subdomain::put(name, &target.account_id, user)
}

pub fn set_subdomain(name: &str, user: &GlobalUser, target: &Target) -> Result<(), failure::Error> {
    if target.account_id.is_empty() {
        failure::bail!(format!(
            "{} You must provide an account_id in your wrangler.toml before creating a subdomain!",
            emoji::WARN
        ))
    }
    let subdomain = Subdomain::get(&target.account_id, user)?;
    if let Some(subdomain) = subdomain {
        let msg = if subdomain == name {
            format!("You have previously registered {}.workers.dev", subdomain)
        } else {
            format!("This account already has a registered subdomain. You can only register one subdomain per account. Your subdomain is {}.workers.dev", subdomain)
        };
        failure::bail!(msg)
    } else {
        register_subdomain(&name, &user, &target)
    }
}

pub fn get_subdomain(user: &GlobalUser, target: &Target) -> Result<(), failure::Error> {
    let subdomain = Subdomain::get(&target.account_id, user)?;
    if let Some(subdomain) = subdomain {
        let msg = format!("{}.workers.dev", subdomain);
        message::info(&msg);
    } else {
        let msg =
            "No subdomain registered. Use `wrangler subdomain <name>` to register one.".to_string();
        message::user_error(&msg);
    }
    Ok(())
}
