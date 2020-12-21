mod edge;
mod gcs;
mod server_config;
mod socket;
mod tls;
mod utils;

pub use server_config::Protocol;
pub use server_config::ServerConfig;

use crate::build::build_target;
use crate::deploy::{self, DeployTarget, DeploymentSet};
use crate::settings::global_user::GlobalUser;
use crate::settings::toml::Target;
use crate::terminal::message::{Message, StdOut};
use crate::terminal::styles;

/// `wrangler dev` starts a server on a dev machine that routes incoming HTTP requests
/// to a Cloudflare Workers runtime and returns HTTP responses
pub fn dev(
    mut target: Target,
    deployments: DeploymentSet,
    user: Option<GlobalUser>,
    server_config: ServerConfig,
    local_protocol: Protocol,
    upstream_protocol: Protocol,
    verbose: bool,
) -> Result<(), failure::Error> {
    // before serving requests we must first build the Worker
    build_target(&target)?;

    if let Some(user) = &user {
        deploy::pre_upload(user, &mut target, &deployments, true)?;
    } else {
        failure::bail!(
            "Previewing a script that binds to a Durable Object namespace is not supported using unauthenticated preview. Please use wrangler login or wrangler config."
        );
    }

    let deploy_target = {
        let valid_targets = deployments
            .into_iter()
            .filter(|t| matches!(t, DeployTarget::Zoned(_) | DeployTarget::Zoneless(_)))
            .collect::<Vec<_>>();

        let valid_target = valid_targets
            .iter()
            .find(|&t| matches!(t, DeployTarget::Zoned(_)))
            .or_else(|| {
                valid_targets
                    .iter()
                    .find(|&t| matches!(t, DeployTarget::Zoneless(_)))
            });

        if let Some(target) = valid_target {
            target.clone()
        } else {
            failure::bail!("No valid deployment targets: `wrangler dev` can only be used to develop zoned and zoneless deployments")
        }
    };

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

    if let Some(user) = user {
        if server_config.host.is_default() {
            // Authenticated and no host provided, run on edge with user's zone
            return edge::dev(
                target,
                user,
                server_config,
                deploy_target,
                local_protocol,
                upstream_protocol,
                verbose,
            );
        }

        // If user is authenticated but host is provided, use gcs with given host
        StdOut::warn(
            format!(
                "{} provided, will run unauthenticated and upstream to provided host",
                host_str
            )
            .as_str(),
        );
    }

    gcs::dev(target, server_config, local_protocol, verbose)
}
