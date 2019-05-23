use crate::user::User;
use reqwest::header::CONTENT_TYPE;
use serde::{Deserialize, Serialize};

use log::info;

#[derive(Deserialize, Serialize)]
pub struct Route {
    script: String,
    pattern: String,
}

#[derive(Deserialize)]
struct RoutesResponse {
    result: Vec<Route>,
}

impl Route {
    pub fn new(user: &User) -> Result<Route, failure::Error> {
        let pattern = &user.settings.clone().project.route.expect(
            "⚠️ Your project config has an error, check your `wrangler.toml`: `route` must be provided.",
        );
        let script = &user.settings.clone().project.name;
        Ok(Route {
            script: script.to_string(),
            pattern: pattern.to_string(),
        })
    }

    pub fn publish(user: &User, route: Route) -> Result<(), failure::Error> {
        if route.exists(user)? {
            return Ok(());
        }
        create(user, route)
    }

    pub fn exists(&self, user: &User) -> Result<bool, failure::Error> {
        let routes = get_routes(user)?;

        for route in routes {
            if route.matches(self) {
                return Ok(true);
            }
        }
        Ok(false)
    }

    pub fn matches(&self, route: &Route) -> bool {
        self.pattern == route.pattern && self.script == route.script
    }
}

fn get_routes(user: &User) -> Result<Vec<Route>, failure::Error> {
    let routes_addr = get_routes_addr(user)?;

    let client = reqwest::Client::new();
    let settings = user.settings.to_owned();

    let mut res = client
        .get(&routes_addr)
        .header("X-Auth-Key", settings.global_user.api_key)
        .header("X-Auth-Email", settings.global_user.email)
        .send()?;

    if !res.status().is_success() {
        let msg = format!(
            "⛔ There was an error featching your project's routes.\n Status Code: {}\n Msg: {}",
            res.status(),
            res.text()?
        );
        failure::bail!(msg)
    }

    let routes_response: RoutesResponse = serde_json::from_str(&res.text()?)?;

    Ok(routes_response.result)
}

fn create(user: &User, route: Route) -> Result<(), failure::Error> {
    let client = reqwest::Client::new();
    let settings = user.settings.to_owned();
    let body = serde_json::to_string(&route)?;

    let routes_addr = get_routes_addr(user)?;

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
    Ok(())
}

fn get_routes_addr(user: &User) -> Result<String, failure::Error> {
    let zone_id = &user.settings.project.zone_id;
    if zone_id.is_empty() {
        failure::bail!("You much provide a zone_id in your wrangler.toml.")
    }
    Ok(format!(
        "https://api.cloudflare.com/client/v4/zones/{}/workers/routes",
        zone_id
    ))
}
