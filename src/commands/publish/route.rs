use crate::http;
use crate::settings::global_user::GlobalUser;
use crate::settings::target::Target;
use crate::terminal::emoji;
use reqwest::header::CONTENT_TYPE;
use serde::{Deserialize, Serialize};

use log::info;

#[derive(Deserialize, Serialize)]
pub struct Route {
    script: Option<String>,
    pub pattern: String,
}

#[derive(Deserialize)]
struct RoutesResponse {
    result: Vec<Route>,
}

impl Route {
    pub fn new(target: &Target) -> Result<Route, failure::Error> {
        if target
            .route
            .clone()
            .expect("You must provide a zone_id in your wrangler.toml before publishing!")
            .is_empty()
        {
            failure::bail!("You must provide a zone_id in your wrangler.toml before publishing!");
        }
        let msg_config_error = format!("{} Your project config has an error, check your `wrangler.toml`: `route` must be provided.", emoji::WARN);
        Ok(Route {
            script: Some(target.name.to_string()),
            pattern: target.route.clone().expect(&msg_config_error),
        })
    }

    pub fn publish(
        user: &GlobalUser,
        target: &Target,
        route: &Route,
    ) -> Result<(), failure::Error> {
        if route.exists(user, target)? {
            return Ok(());
        }
        create(user, target, route)
    }

    pub fn exists(&self, user: &GlobalUser, target: &Target) -> Result<bool, failure::Error> {
        let routes = get_routes(user, target)?;

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

fn get_routes(user: &GlobalUser, target: &Target) -> Result<Vec<Route>, failure::Error> {
    let routes_addr = get_routes_addr(target)?;

    let client = http::auth_client(user);

    let mut res = client.get(&routes_addr).send()?;

    if !res.status().is_success() {
        let msg = format!(
            "{} There was an error fetching your project's routes.\n Status Code: {}\n Msg: {}",
            emoji::WARN,
            res.status(),
            res.text()?
        );
        failure::bail!(msg)
    }

    let routes_response: RoutesResponse = serde_json::from_str(&res.text()?)?;

    Ok(routes_response.result)
}

fn create(user: &GlobalUser, target: &Target, route: &Route) -> Result<(), failure::Error> {
    let client = http::auth_client(user);
    let body = serde_json::to_string(&route)?;

    let routes_addr = get_routes_addr(target)?;

    info!("Creating your route {:#?}", &route.pattern,);
    let mut res = client
        .post(&routes_addr)
        .header(CONTENT_TYPE, "application/json")
        .body(body)
        .send()?;

    if !res.status().is_success() {
        let msg = format!(
            "{} There was an error creating your route.\n Status Code: {}\n Msg: {}",
            emoji::WARN,
            res.status(),
            res.text()?
        );
        failure::bail!(msg)
    }
    Ok(())
}

fn get_routes_addr(target: &Target) -> Result<String, failure::Error> {
    if let Some(zone_id) = &target.zone_id {
        return Ok(format!(
            "https://api.cloudflare.com/client/v4/zones/{}/workers/routes",
            zone_id
        ));
    }
    failure::bail!("You much provide a zone_id in your wrangler.toml.")
}
