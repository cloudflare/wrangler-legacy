mod edge;
mod gcs;
mod server_config;
mod socket;
mod utils;

use server_config::ServerConfig;

use crate::build;
use crate::settings::global_user::GlobalUser;
use crate::settings::toml::{DeployConfig, Target};

/// `wrangler dev` starts a server on a dev machine that routes incoming HTTP requests
/// to a Cloudflare Workers runtime and returns HTTP responses
pub fn dev(
    target: Target,
    deploy_config: DeployConfig,
    user: Option<GlobalUser>,
    host: Option<&str>,
    port: Option<u16>,
    ip: Option<&str>,
    verbose: bool,
) -> Result<(), failure::Error> {
    let server_config = ServerConfig::new(host, ip, port)?;

    // before serving requests we must first build the Worker
    build(&target)?;

    match user {
        // authenticated users connect to the edge
        Some(user) => edge::dev(target, user, server_config, deploy_config, verbose),

        // unauthenticated users connect to gcs
        None => gcs::dev(target, server_config, verbose),
    }
}
