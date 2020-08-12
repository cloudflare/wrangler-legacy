mod edge;
mod gcs;
mod server_config;
mod socket;
mod tls;
mod utils;

pub use server_config::Protocol;
pub use server_config::ServerConfig;

use crate::build;
use crate::settings::global_user::GlobalUser;
use crate::settings::toml::{DeployConfig, Target};
use crate::terminal::styles;

/// `wrangler dev` starts a server on a dev machine that routes incoming HTTP requests
/// to a Cloudflare Workers runtime and returns HTTP responses
pub fn dev(
    target: Target,
    deploy_config: DeployConfig,
    user: Option<GlobalUser>,
    server_config: ServerConfig,
    local_protocol: Protocol,
    upstream_protocol: Protocol,
    verbose: bool,
) -> Result<(), failure::Error> {
    // before serving requests we must first build the Worker
    build(&target)?;

    let host_str = styles::highlight("--host");
    let local_str = styles::highlight("--local-protocol");
    let upstream_str = styles::highlight("--upstream-protocol");

    if server_config.host.is_https() != upstream_protocol.is_https() {
        failure::bail!(format!(
            "Protocol mismatch: protocol in {} and protocol in {} must match",
            host_str, upstream_str
        ))
    } else if local_protocol.is_https() && upstream_protocol.is_http() {
        failure::bail!("{} cannot be https if {} is http", local_str, upstream_str)
    }

    match user {
        // authenticated users connect to the edge
        Some(user) => edge::dev(
            target,
            user,
            server_config,
            deploy_config,
            local_protocol,
            upstream_protocol,
            verbose,
        ),

        // unauthenticated users connect to gcs
        None => gcs::dev(target, server_config, local_protocol, verbose),
    }
}
