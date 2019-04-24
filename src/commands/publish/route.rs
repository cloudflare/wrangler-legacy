use crate::user::User;
use reqwest::header::CONTENT_TYPE;
use serde::Serialize;

#[derive(Serialize)]
pub struct Route {
    #[serde(skip_serializing_if = "Option::is_none")]
    enabled: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    script: Option<String>,
    pattern: String,
}

impl Route {
    pub fn create(user: &User, script: Option<String>) -> Result<Route, failure::Error> {
        if user.account.multiscript {
            match script {
                Some(s) => multi_script(user, s),
                None => failure::bail!("⚠️ You must provide the name of the script you'd like to associate with this route."),
            }
        } else {
            if script.is_some() {
                println!("⚠️ You only have a single script account. Ignoring name.");
            }
            single_script(user)
        }
    }
}

fn multi_script(user: &User, script: String) -> Result<Route, failure::Error> {
    let pattern = &user.settings.clone().project.route.expect(
        "⚠️ Your project config has an error, check your `wrangler.toml`: `route` must be provided.",
    );
    let route = Route {
        script: Some(script),
        pattern: pattern.to_string(),
        enabled: None,
    };
    let zone_id = &user.settings.project.zone_id;
    let routes_addr = format!(
        "https://api.cloudflare.com/client/v4/zones/{}/workers/routes",
        zone_id
    );

    let client = reqwest::Client::new();
    let settings = user.settings.to_owned();
    let body = serde_json::to_string(&route)?;

    let mut res = client
        .post(&routes_addr)
        .header("X-Auth-Key", settings.global_user.api_key)
        .header("X-Auth-Email", settings.global_user.email)
        .header(CONTENT_TYPE, "application/json")
        .body(body)
        .send()?;

    if !res.status().is_success() {
        let msg = format!(
            "⛔ There was an error creating your route.\n Status Code: {}\n Msg: {}",
            res.status(),
            res.text()?
        );
        failure::bail!(msg)
    }
    Ok(route)
}

fn single_script(user: &User) -> Result<Route, failure::Error> {
    let pattern = user.settings.clone().project.route.expect(
        "⚠️ Your project config has an error, check your `wrangler.toml`: `route` must be provided.",
    );
    let route = Route {
        script: None,
        pattern,
        enabled: Some(true),
    };
    let zone_id = &user.settings.project.zone_id;
    let filters_addr = format!(
        "https://api.cloudflare.com/client/v4/zones/{}/workers/filters",
        zone_id
    );

    let client = reqwest::Client::new();
    let settings = user.settings.to_owned();

    client
        .put(&filters_addr)
        .header("X-Auth-Key", settings.global_user.api_key)
        .header("X-Auth-Email", settings.global_user.email)
        .header(CONTENT_TYPE, "application/json")
        .body(serde_json::to_string(&route)?)
        .send()?;

    Ok(route)
}
