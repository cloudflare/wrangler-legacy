extern crate serde_json;

use cloudflare::endpoints::workers::{DeleteRoute, ListRoutes};
use cloudflare::framework::apiclient::ApiClient;

use crate::http;
use crate::settings::global_user::GlobalUser;
use crate::terminal::message;

pub fn list(zone_identifier: String, user: &GlobalUser) -> Result<(), failure::Error> {
    let client = http::cf_api_client(user, http::CfApiClientConfig::default())?;

    let result = client.request(&ListRoutes {
        zone_identifier: &zone_identifier,
    });

    match result {
        Ok(success) => {
            let routes = success.result;
            println!("{}", serde_json::to_string(&routes)?);
        }

        Err(e) => failure::bail!("{}", http::format_error(e, None)),
    }
    Ok(())
}

pub fn delete(
    zone_identifier: String,
    user: &GlobalUser,
    route_id: &str,
) -> Result<(), failure::Error> {
    let client = http::cf_api_client(user, http::CfApiClientConfig::default())?;

    let result = client.request(&DeleteRoute {
        zone_identifier: &zone_identifier,
        identifier: route_id,
    });

    match result {
        Ok(success) => {
            let msg = format!("Successfully deleted route with id {}", success.result.id);
            message::success(&msg);
        }

        Err(e) => failure::bail!("{}", http::format_error(e, Some(&error_suggestions))),
    }
    Ok(())
}

fn error_suggestions(code: u16) -> &'static str {
    match code {
        10005 => "Confirm the route id by running `wrangler route list`",
        _ => "",
    }
}
