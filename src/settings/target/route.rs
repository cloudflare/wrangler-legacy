use serde::{Deserialize, Serialize};

use cloudflare::endpoints::workers::WorkersRoute;

use crate::settings::target::target::Target;
use crate::terminal::emoji;

#[derive(Debug, Deserialize, PartialEq, Serialize)]
pub struct Route {
    pub script: Option<String>,
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
