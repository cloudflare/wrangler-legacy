use super::settings::Settings;
use reqwest::header::CONTENT_TYPE;
use serde::{self, Deserialize};

#[derive(Debug, Deserialize)]
struct AccountResponse {
    pub success: bool,
    pub errors: Vec<String>,
    pub result: Vec<AccountData>,
}

#[derive(Debug, Deserialize)]
struct AccountData {
    pub id: String,
    pub name: String,
}

#[derive(Debug, Deserialize)]
pub struct Account {
    pub id: String,
    pub name: String,
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
        let account = account_data(settings.clone())?;
        let multiscript = script_status(settings, &account)?;
        Ok(Account {
            id: account.id,
            name: account.name,
            multiscript,
        })
    }
}

fn account_data(settings: Settings) -> Result<AccountData, failure::Error> {
    let user_addr = "https://api.cloudflare.com/client/v4/accounts";
    let client = reqwest::Client::new();

    let mut res = client
        .get(user_addr)
        .header("X-Auth-Key", settings.api_key)
        .header("X-Auth-Email", settings.email)
        .header(CONTENT_TYPE, "application/json")
        .send()?;

    let mut account_res: AccountResponse = serde_json::from_str(&res.text()?)?;
    let account: AccountData = account_res.result.remove(0); // TODO(ashleygwilliams): this should be from per project config
    Ok(account)
}

fn script_status(settings: Settings, account_data: &AccountData) -> Result<bool, failure::Error> {
    let addr = format!(
        "https://api.cloudflare.com/client/v4/accounts/{}/workers/settings",
        account_data.id
    );
    let client = reqwest::Client::new();

    let mut res = client
        .get(&addr)
        .header("X-Auth-Key", settings.api_key)
        .header("X-Auth-Email", settings.email)
        .header(CONTENT_TYPE, "application/json")
        .send()?;

    let status: ScriptsResponse = serde_json::from_str(&res.text()?)?;
    Ok(status.result.multiscript)
}
