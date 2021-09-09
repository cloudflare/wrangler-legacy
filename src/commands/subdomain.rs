use crate::http;
use crate::settings::global_user::GlobalUser;
use crate::settings::toml::Target;
use crate::terminal::message::{Message, StdOut};
use crate::terminal::{emoji, interactive};

use anyhow::Result;
use serde::{Deserialize, Serialize};

#[derive(Serialize)]
pub struct Subdomain {
    subdomain: String,
}

impl Subdomain {
    pub fn get(account_id: &str, user: &GlobalUser) -> Result<Option<String>> {
        let addr = subdomain_addr(account_id);

        let client = http::legacy_auth_client(user);

        let response = client.get(&addr).send()?;

        if !response.status().is_success() {
            anyhow::bail!(
                "{} There was an error fetching your subdomain.\n Status Code: {}\n Msg: {}",
                emoji::WARN,
                response.status(),
                response.text()?,
            )
        }
        let response: SubdomainResponse = serde_json::from_str(&response.text()?)?;
        Ok(response.result.map(|r| r.subdomain))
    }

    pub fn put(name: &str, account_id: &str, user: &GlobalUser) -> Result<()> {
        let addr = subdomain_addr(account_id);
        let subdomain = Subdomain {
            subdomain: name.to_string(),
        };
        let subdomain_request = serde_json::to_string(&subdomain)?;

        let client = http::legacy_auth_client(user);

        let response = client
            .put(&addr)
            .header("allow-rename", "1")
            .body(subdomain_request)
            .send()?;

        let response_status = response.status();
        if !response_status.is_success() {
            let response_text = &response.text()?;
            let r: SubdomainResponse = serde_json::from_str(response_text)?;
            let api_error = r.errors.first().unwrap();
            log::debug!("Status Code: {}", response_status);
            log::debug!("Status Message: {}", response_text);
            let msg = if response_status == 403 && api_error.code == 10031 {
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
            anyhow::bail!(msg)
        }
        StdOut::success(&format!("Success! You've registered {}.", name));
        Ok(())
    }
}

#[derive(Deserialize)]
struct SubdomainResponse {
    result: Option<SubdomainResult>,
    errors: Vec<Error>,
}

#[derive(Deserialize)]
struct SubdomainResult {
    subdomain: String,
}

#[derive(Deserialize)]
struct ScriptResponse {
    result: Vec<ScriptResult>,
}

#[derive(Deserialize)]
struct ScriptResult {
    id: String,
    available_on_subdomain: bool,
}

#[derive(Deserialize)]
struct Error {
    code: i64,
}

fn subdomain_addr(account_id: &str) -> String {
    format!(
        "https://api.cloudflare.com/client/v4/accounts/{}/workers/subdomain",
        account_id
    )
}

fn register_subdomain(name: &str, user: &GlobalUser, target: &Target) -> Result<()> {
    let msg = format!(
        "Registering your subdomain, {}.workers.dev, this could take up to a minute.",
        name
    );
    StdOut::working(&msg);
    Subdomain::put(name, target.account_id.load()?, user)
}

pub fn set_subdomain(name: &str, user: &GlobalUser, target: &Target) -> Result<()> {
    let account_id = target.account_id.load()?;
    let subdomain = Subdomain::get(account_id, user)?;
    if let Some(subdomain) = subdomain {
        if subdomain == name {
            let msg = format!("You have already registered {}.workers.dev", subdomain);
            StdOut::success(&msg);
            return Ok(());
        } else {
            // list all the affected scripts
            let scripts = get_subdomain_scripts(account_id, user)?;

            let default_msg = format!("Are you sure you want to permanently move your subdomain from {}.workers.dev to {}.workers.dev?",
                                      subdomain, name);
            let prompt_msg = if scripts.is_empty() {
                default_msg
            } else {
                let mut script_updates: Vec<String> = Vec::new();
                for script in scripts {
                    script_updates.push(format!(
                        "{}.{}.workers.dev => {}.{}.workers.dev",
                        script, subdomain, script, name
                    ))
                }
                let msg = format!(
                    "The following deployed Workers will be affected:\n{}\nIt may take a few minutes for these Workers to become available again.",
                    script_updates.join("\n")
                );
                format!("{}\n{}", msg, default_msg)
            };

            match interactive::confirm(&prompt_msg) {
                Ok(true) => (),
                Ok(false) => {
                    StdOut::info(&format!("Keeping subdomain: {}.workers.dev", subdomain));
                    return Ok(());
                }
                Err(e) => anyhow::bail!(e),
            }
        }
    }

    register_subdomain(name, user, target)
}

pub fn get_subdomain(user: &GlobalUser, target: &Target) -> Result<()> {
    let subdomain = Subdomain::get(target.account_id.load()?, user)?;
    if let Some(subdomain) = subdomain {
        let msg = format!("{}.workers.dev", subdomain);
        StdOut::info(&msg);
    } else {
        let msg =
            "No subdomain registered. Use `wrangler subdomain <name>` to register one.".to_string();
        StdOut::user_error(&msg);
    }
    Ok(())
}

fn get_subdomain_scripts(account_id: &str, user: &GlobalUser) -> Result<Vec<String>> {
    let addr = scripts_addr(account_id);

    let client = http::legacy_auth_client(user);

    let response = client
        .get(&addr)
        .query(&[("include_subdomain_availability", "1")])
        .send()?;

    if !response.status().is_success() {
        anyhow::bail!(
            "{} There was an error fetching scripts.\n Status Code: {}\n Msg: {}",
            emoji::WARN,
            response.status(),
            response.text()?,
        )
    }
    let response: ScriptResponse = serde_json::from_str(&response.text()?)?;
    let mut scripts: Vec<String> = Vec::new();
    for script in response.result {
        if script.available_on_subdomain {
            scripts.push(script.id)
        }
    }
    Ok(scripts)
}

fn scripts_addr(account_id: &str) -> String {
    format!(
        "https://api.cloudflare.com/client/v4/accounts/{}/workers/scripts",
        account_id
    )
}
