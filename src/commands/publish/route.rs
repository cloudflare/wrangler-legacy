use cloudflare::endpoints::workers::{CreateRoute, CreateRouteParams, ListRoutes};
use cloudflare::framework::apiclient::ApiClient;
use cloudflare::framework::HttpApiClientConfig;

use crate::http::{api_client, format_error};
use crate::settings::global_user::GlobalUser;
use crate::settings::target::{Route, Target};

pub fn publish_route(user: &GlobalUser, target: &Target) -> Result<String, failure::Error> {
    let route = Route::new(&target)?;
    if route_exists(user, target, &route)? {
        Ok(route.pattern)
    } else {
        create(user, target, &route)?;
        Ok(route.pattern)
    }
}

fn route_exists(user: &GlobalUser, target: &Target, route: &Route) -> Result<bool, failure::Error> {
    let routes = fetch_all(user, target)?;

    for remote_route in routes {
        if remote_route == *route {
            return Ok(true);
        }
    }
    Ok(false)
}

fn fetch_all(user: &GlobalUser, target: &Target) -> Result<Vec<Route>, failure::Error> {
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
