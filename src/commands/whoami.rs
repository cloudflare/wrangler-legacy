use crate::settings::Settings;
use reqwest::header::CONTENT_TYPE;
use serde::{self, Deserialize};

#[derive(Debug, Deserialize)]
struct CFUserResponse {
    result: CFUser,
}

#[derive(Debug, Deserialize)]
struct CFUser {
    pub email: String,
    pub has_enterprise_zones: bool,
    pub has_pro_zones: bool,
    pub has_business_zones: bool,
    suspended: bool,
    pub enterprise_zone_quota: CFQuota,
}

fn emoji(b: bool) -> String {
    match b {
        true => "✅".to_string(),
        false => "⛔".to_string(),
    }
}

#[derive(Debug, Deserialize)]
struct CFQuota {
    pub maximum: i32,
    current: i32,
    pub available: i32,
}

pub fn whoami(settings: Settings) -> Result<(), failure::Error> {
    let user_addr = "https://api.cloudflare.com/client/v4/user";

    let client = reqwest::Client::new();

    let mut res = client
        .get(user_addr)
        .header("X-Auth-Key", settings.api_key)
        .header("X-Auth-Email", settings.email)
        .header(CONTENT_TYPE, "application/json")
        .send()?;

    let user: CFUserResponse = serde_json::from_str(&res.text()?)?;
    let user = user.result;
    println!("👋 You are logged in as {}.", user.email);
    println!("{} Enterprise | {} Business | {} Pro", &emoji(user.has_enterprise_zones), &emoji(user.has_business_zones), &emoji(user.has_pro_zones));
    if user.has_enterprise_zones {
        println!("🏘️  {} of {} Enterprise Zones are available.", user.enterprise_zone_quota.available, user.enterprise_zone_quota.maximum);
    }
    Ok(())
}
