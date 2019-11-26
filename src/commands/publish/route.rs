use serde::{Deserialize, Serialize};

use cloudflare::endpoints::workers::{CreateRoute, CreateRouteParams, ListRoutes, WorkersRoute};
use cloudflare::framework::apiclient::ApiClient;
use cloudflare::framework::HttpApiClientConfig;

use crate::http::{api_client, format_error};
use crate::settings::global_user::GlobalUser;
use crate::settings::target::Target;
use crate::terminal::emoji;

#[derive(Deserialize, PartialEq, Serialize)]
pub struct Route {
    script: Option<String>,
    pub pattern: String,
}

impl From<&WorkersRoute> for Route {
    fn from(api_route: &WorkersRoute) -> Route {
        Route {
            script: api_route.script.clone(),
            pattern: api_route.pattern.clone(),
        }
    }
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
        let msg_config_error = format!(
            "{} Your project config has an error, check your `wrangler.toml`: `route` must be provided.", 
            emoji::WARN
        );
        Ok(Route {
            script: Some(target.name.to_string()),
            pattern: target.route.clone().expect(&msg_config_error),
        })
    }
}

pub fn publish(user: &GlobalUser, target: &Target, route: &Route) -> Result<(), failure::Error> {
    if exists(user, target, route)? {
        return Ok(());
    }
    create(user, target, route)
}

fn exists(user: &GlobalUser, target: &Target, route: &Route) -> Result<bool, failure::Error> {
    let routes = get_routes(user, target)?;

    for remote_route in routes {
        if remote_route == *route {
            return Ok(true);
        }
    }
    Ok(false)
}

fn get_routes(user: &GlobalUser, target: &Target) -> Result<Vec<Route>, failure::Error> {
    let client = api_client(user, HttpApiClientConfig::default())?;

    let routes: Vec<Route> = match client.request(&ListRoutes {
        zone_identifier: &target.zone_id.as_ref().unwrap(),
    }) {
        Ok(success) => success.result.iter().map(|r| Route::from(r)).collect(),
        Err(e) => failure::bail!("{}", format_error(e, None)), // TODO: add suggestion fn
    };

    Ok(routes)
}

fn create(user: &GlobalUser, target: &Target, route: &Route) -> Result<(), failure::Error> {
    let client = api_client(user, HttpApiClientConfig::default())?;

    log::info!("Creating your route {:#?}", &route.pattern,);
    match client.request(&CreateRoute {
        zone_identifier: &target.zone_id.as_ref().unwrap(),
        params: CreateRouteParams {
            pattern: route.pattern.clone(),
            script: route.script.clone(),
        },
    }) {
        Ok(_success) => Ok(()),
        Err(e) => failure::bail!("{}", format_error(e, Some(&routes_error_help))),
    }
}

fn routes_error_help(error_code: u16) -> &'static str {
    match error_code {
        10020 => r#"
            A worker with a different name was previously deployed to the specified route.
            If you would like to overwrite that worker,
            you will need to change `name` in your `wrangler.toml` to match the currently deployed worker,
            or navigate to https://dash.cloudflare.com/workers and rename or delete that worker.\n"#,
        _ => "",
    }
}
