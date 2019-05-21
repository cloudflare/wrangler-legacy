use crate::user::User;
use reqwest::header::CONTENT_TYPE;
use serde::Serialize;

use log::info;

#[derive(Serialize)]
pub struct Route {
    script: String,
    pattern: String,
}

impl Route {
    pub fn create(user: &User, script: String) -> Result<Route, failure::Error> {
        create(user, script)
    }
}

fn create(user: &User, script: String) -> Result<Route, failure::Error> {
    let pattern = &user.settings.clone().project.route.expect(
        "⚠️ Your project config has an error, check your `wrangler.toml`: `route` must be provided.",
    );
    let route = Route {
        script,
        pattern: pattern.to_string(),
    };
    let zone_id = &user.settings.project.zone_id;
    if zone_id.is_empty() {
        failure::bail!("You much provide a zone_id in your wrangler.toml.")
    }
    let routes_addr = format!(
        "https://api.cloudflare.com/client/v4/zones/{}/workers/routes",
        zone_id
    );

    let client = reqwest::Client::new();
    let settings = user.settings.to_owned();
    let body = serde_json::to_string(&route)?;

    info!(
        "Creating your route {} for script {}",
        route.pattern, route.script
    );
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
