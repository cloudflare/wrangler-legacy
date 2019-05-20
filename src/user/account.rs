use super::settings::Settings;
use reqwest::header::CONTENT_TYPE;
use serde::{self, Deserialize};

use log::info;

#[derive(Debug, Deserialize)]
pub struct Account {
    pub multiscript: bool,
}

#[derive(Debug, Deserialize)]
struct ScriptsResponse {
    pub result: AccountScripts,
}

#[derive(Debug, Deserialize)]
struct AccountScripts {
    pub multiscript: bool,
}

impl Account {
    pub fn new(settings: Settings) -> Result<Account, failure::Error> {
        let multiscript = script_status(settings)?;
        Ok(Account { multiscript })
    }
}

fn script_status(settings: Settings) -> Result<bool, failure::Error> {
    info!("Requesting user's script status...");

    let addr = format!(
        "https://api.cloudflare.com/client/v4/accounts/{}/workers/settings",
        settings.project.account_id
    );
    let client = reqwest::Client::new();

    let mut res = client
        .get(&addr)
        .header("X-Auth-Key", settings.global_user.api_key)
        .header("X-Auth-Email", settings.global_user.email)
        .header(CONTENT_TYPE, "application/json")
        .send()?;

    let status: ScriptsResponse = serde_json::from_str(&res.text()?)?;
    Ok(status.result.multiscript)
}
