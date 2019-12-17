use serde::Serialize;

use cloudflare::endpoints::workers::{CreateRoute, CreateRouteParams, ListRoutes};
use cloudflare::framework::apiclient::ApiClient;
use cloudflare::framework::HttpApiClientConfig;

use crate::http::{cf_v4_api_client, format_error};
use crate::settings::global_user::GlobalUser;
use crate::settings::toml::{Route, Zoned};

pub fn publish_routes(
    user: &GlobalUser,
    zoned_config: &Zoned,
) -> Result<Vec<RouteUploadResult>, failure::Error> {
    // For the moment, we'll just make this call once and make all our decisions based on the response.
    // There is a possibility of race conditions, but we just report back the results and allow the
    // user to decide how to procede.
    let existing_routes = fetch_all(user, &zoned_config.zone_id)?;

    let deployed_routes = zoned_config
        .routes
        .iter()
        .map(|route| deploy_route(user, &zoned_config.zone_id, route, &existing_routes))
        .collect();

    Ok(deployed_routes)
}

fn fetch_all(user: &GlobalUser, zone_identifier: &str) -> Result<Vec<Route>, failure::Error> {
    let client = cf_v4_api_client(user, HttpApiClientConfig::default())?;

    let routes: Vec<Route> = match client.request(&ListRoutes { zone_identifier }) {
        Ok(success) => success.result.iter().map(Route::from).collect(),
        Err(e) => failure::bail!("{}", format_error(e, None)), // TODO: add suggestion fn
    };

    Ok(routes)
}

fn create(
    user: &GlobalUser,
    zone_identifier: &str,
    route: &Route,
) -> Result<Route, failure::Error> {
    let client = cf_v4_api_client(user, HttpApiClientConfig::default())?;

    log::info!("Creating your route {:#?}", &route.pattern,);
    match client.request(&CreateRoute {
        zone_identifier,
        params: CreateRouteParams {
            pattern: route.pattern.clone(),
            script: route.script.clone(),
        },
    }) {
        Ok(response) => Ok(Route {
            id: Some(response.result.id),
            pattern: route.pattern.clone(),
            script: route.script.clone(),
        }),
        Err(e) => failure::bail!("{}", format_error(e, Some(&routes_error_help))),
    }
}

// TODO: improve this error message to reference wrangler route commands
fn routes_error_help(error_code: u16) -> &'static str {
    match error_code {
        10020 => {
            r#"
            A worker with a different name was previously deployed to the specified route.
            If you would like to overwrite that worker,
            you will need to change `name` in your `wrangler.toml` to match the currently deployed worker,
            or navigate to https://dash.cloudflare.com/workers and rename or delete that worker.\n"#
        }
        _ => "",
    }
}

#[derive(Debug, Serialize)]
pub enum RouteUploadResult {
    Same(Route),
    Conflict(Route),
    New(Route),
    Error((Route, String)),
}

fn deploy_route(
    user: &GlobalUser,
    zone_id: &str,
    route: &Route,
    existing_routes: &[Route],
) -> RouteUploadResult {
    for existing_route in existing_routes {
        if route.pattern == existing_route.pattern {
            // if the route is already assigned, we don't need to call the api.
            // if the script names match, it's a no-op.
            if route.script == existing_route.script {
                return RouteUploadResult::Same(Route {
                    id: existing_route.id.clone(),
                    script: existing_route.script.clone(),
                    pattern: existing_route.pattern.clone(),
                });
            }
            // if the script names do not match, we want to know which script is conflicting.
            return RouteUploadResult::Conflict(Route {
                id: existing_route.id.clone(),
                script: existing_route.script.clone(),
                pattern: existing_route.pattern.clone(),
            });
        }
    }

    // if none of the existing routes match this one, we should create a new route
    match create(user, zone_id, &route) {
        // we want to show the new route along with its id
        Ok(created) => RouteUploadResult::New(created),
        // if there is an error, we want to know which route triggered it
        Err(e) => RouteUploadResult::Error((
            Route {
                id: None,
                script: route.script.clone(),
                pattern: route.pattern.clone(),
            },
            e.to_string(),
        )),
    }
}
