extern crate serde_json;

use crate::http;

use cloudflare::endpoints::workers::{DeleteRoute, ListRoutes};
use cloudflare::framework::apiclient::ApiClient;
use cloudflare::framework::HttpApiClientConfig;

use crate::settings::global_user::GlobalUser;
use crate::settings::toml::Target;
use crate::terminal::message;

pub fn list(target: &Target, user: &GlobalUser) -> Result<(), failure::Error> {
    let client = http::cf_v4_api_client(user, HttpApiClientConfig::default())?;

    let result = client.request(&ListRoutes {
        zone_identifier: &target
            .zone_id
            .as_ref()
            .expect("missing zone_id in `wrangler.toml`"),
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

pub fn delete(target: &Target, user: &GlobalUser, route_id: &str) -> Result<(), failure::Error> {
    let client = http::cf_v4_api_client(user, HttpApiClientConfig::default())?;

    let result = client.request(&DeleteRoute {
        zone_identifier: &target
            .zone_id
            .as_ref()
            .expect("missing zone_id in `wrangler.toml`"),
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
