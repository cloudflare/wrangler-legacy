use anyhow::Result;
use cloudflare::endpoints::workers::{DeleteRoute, ListRoutes};
use cloudflare::framework::apiclient::ApiClient;

use crate::http;
use crate::settings::global_user::GlobalUser;
use crate::terminal::message::{Message, StdOut};

pub fn list(zone_identifier: &str, user: &GlobalUser) -> Result<()> {
    let client = http::cf_v4_client(user)?;

    let result = client.request(&ListRoutes { zone_identifier });

    match result {
        Ok(success) => {
            let routes = success.result;
            println!("{}", serde_json::to_string(&routes)?);
        }

        Err(e) => anyhow::bail!("{}", http::format_error(e, None)),
    }
    Ok(())
}

pub fn delete(zone_identifier: &str, user: &GlobalUser, route_id: &str) -> Result<()> {
    let client = http::cf_v4_client(user)?;

    let result = client.request(&DeleteRoute {
        zone_identifier: &zone_identifier,
        identifier: route_id,
    });

    match result {
        Ok(success) => {
            let msg = format!("Successfully deleted route with id {}", success.result.id);
            StdOut::success(&msg);
        }

        Err(e) => anyhow::bail!("{}", http::format_error(e, Some(&error_suggestions))),
    }
    Ok(())
}

fn error_suggestions(code: u16) -> &'static str {
    match code {
        10005 => "Confirm the route id by running `wrangler route list`",
        _ => "",
    }
}
