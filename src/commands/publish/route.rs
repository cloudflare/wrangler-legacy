use cloudflare::endpoints::workers::{CreateRoute, CreateRouteParams, ListRoutes};
use cloudflare::framework::apiclient::ApiClient;
use cloudflare::framework::HttpApiClientConfig;

use crate::http::{api_client, format_error};
use crate::settings::global_user::GlobalUser;
use crate::settings::target::Route;

pub fn publish_routes(
    user: &GlobalUser,
    routes: Vec<Route>,
    zone_id: &String,
) -> Result<Vec<Route>, failure::Error> {
    // get existing routes
    let existing_routes = fetch_all(user, zone_id)?;

    let mut deployed_routes = Vec::new();

    for route in routes {
        if existing_routes.contains(&route) {
            deployed_routes.push(route);
        } else {
            match create(user, zone_id, &route) {
                Ok(_) => deployed_routes.push(route),
                Err(e) => failure::bail!(
                    "An error occurred deploying to route {}: {}",
                    route.pattern,
                    e
                ),
            }
        }
    }

    // TODO: include id info in Route objects
    Ok(deployed_routes)
}

fn fetch_all(user: &GlobalUser, zone_identifier: &String) -> Result<Vec<Route>, failure::Error> {
    let client = api_client(user, HttpApiClientConfig::default())?;

    let routes: Vec<Route> = match client.request(&ListRoutes { zone_identifier }) {
        Ok(success) => success.result.iter().map(|r| Route::from(r)).collect(),
        Err(e) => failure::bail!("{}", format_error(e, None)), // TODO: add suggestion fn
    };

    // TODO: include id info in Route objects
    Ok(routes)
}

// TODO: merge id info into returned Route object
fn create(
    user: &GlobalUser,
    zone_identifier: &String,
    route: &Route,
) -> Result<(), failure::Error> {
    let client = api_client(user, HttpApiClientConfig::default())?;

    log::info!("Creating your route {:#?}", &route.pattern,);
    match client.request(&CreateRoute {
        zone_identifier,
        params: CreateRouteParams {
            pattern: route.pattern.clone(),
            script: route.script.clone(),
        },
    }) {
        Ok(_) => Ok(()),
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
