use super::Cli;
use crate::commands;
use crate::settings::{global_user::GlobalUser, toml::Manifest};

use anyhow::Result;
use structopt::StructOpt;

#[derive(Debug, Clone, StructOpt)]
#[structopt(rename_all = "lower")]
pub enum Route {
    /// List all routes associated with a zone (outputs json)
    List,
    /// Delete a route by ID
    Delete {
        /// The ID associated with the route you want to delete (find using `wrangler route list`)
        #[structopt(index = 1)]
        route_id: String,
    },
}

pub fn route(route: Route, cli_params: &Cli) -> Result<()> {
    let user = GlobalUser::new()?;
    let manifest = Manifest::new(&cli_params.config)?;
    let zone_id = manifest
        .get_environment(cli_params.environment.as_deref())?
        .and_then(|e| e.zone_id.as_ref())
        .or_else(|| manifest.zone_id.as_ref());

    let zone_id = zone_id.ok_or_else(|| {
        anyhow::anyhow!(
        "You must specify a zone_id in your configuration file to use `wrangler route` commands."
    )
    })?;

    match route {
        Route::List => commands::route::list(zone_id, &user),
        Route::Delete { route_id } => commands::route::delete(zone_id, &user, &route_id),
    }
}
