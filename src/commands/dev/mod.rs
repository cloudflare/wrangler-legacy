mod edge;
mod gcs;
mod server_config;
mod socket;
mod tls;
mod utils;

pub use server_config::ServerConfig;

use crate::build;
use crate::settings::global_user::GlobalUser;
use crate::settings::toml::{DeployConfig, Target};

/// `wrangler dev` starts a server on a dev machine that routes incoming HTTP requests
/// to a Cloudflare Workers runtime and returns HTTP responses
pub fn dev(
    target: Target,
    deploy_config: DeployConfig,
    user: Option<GlobalUser>,
    server_config: ServerConfig,
    http: bool,
    verbose: bool,
) -> Result<(), failure::Error> {
    // before serving requests we must first build the Worker
    build(&target)?;

    match user {
        // authenticated users connect to the edge
        Some(user) => edge::dev(target, user, server_config, deploy_config, http, verbose),

        // unauthenticated users connect to gcs
        None => {
            if http {
                failure::bail!("Unauthenticated dev must use https")
            } else {
                gcs::dev(target, server_config, verbose)
            }
        }
    }
}
