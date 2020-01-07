extern crate serde_json;

use cloudflare::endpoints::workers::{DeleteRoute, ListRoutes};
use cloudflare::framework::apiclient::ApiClient;
use cloudflare::framework::HttpApiClientConfig;

use crate::http;
use crate::settings::global_user::GlobalUser;
use crate::terminal::message;

pub fn list(zone_identifier: String, user: &GlobalUser) -> Result<(), failure::Error> {
    let client = http::cf_v4_api_client(user, HttpApiClientConfig::default())?;

    let result = client.request(&ListRoutes {
        zone_identifier: &zone_identifier,
    });

    match result {
        Ok(success) => {
            let routes = success.result;
            println!("{}", serde_json::to_string(&routes)?);
        }
        // TODO: handle route errors with http::format_error
        Err(e) => failure::bail!("{}", e),
    }
    Ok(())
}

pub fn delete(
    zone_identifier: String,
    user: &GlobalUser,
    route_id: &str,
) -> Result<(), failure::Error> {
    let client = http::cf_v4_api_client(user, HttpApiClientConfig::default())?;

    let result = client.request(&DeleteRoute {
        zone_identifier: &zone_identifier,
        identifier: route_id,
    });

    match result {
        Ok(success) => {
            let msg = format!("Successfully deleted route with id {}", success.result.id);
            message::success(&msg);
        }
        // TODO: handle route errors with http::format_error
        Err(e) => failure::bail!("{}", e),
    }
    Ok(())
}
