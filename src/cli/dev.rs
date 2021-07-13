use std::net::{IpAddr, Ipv4Addr};

use super::Cli;
use crate::commands::{self, dev::Protocol};
use crate::settings::{global_user::GlobalUser, toml::Manifest};

use anyhow::Result;

pub fn dev(
    host: Option<String>,
    mut ip: Option<IpAddr>,
    mut port: Option<u16>,
    mut local_protocol: Option<Protocol>,
    mut upstream_protocol: Option<Protocol>,
    cli_params: &Cli,
    inspect: bool,
) -> Result<()> {
    log::info!("Starting dev server");
    let manifest = Manifest::new(&cli_params.config)?;

    // Check if arg not given but present in wrangler.toml
    if let Some(d) = &manifest.dev {
        ip = ip.or(d.ip);
        port = port.or(d.port);
        local_protocol = local_protocol.or(d.local_protocol);
        upstream_protocol = upstream_protocol.or(d.upstream_protocol);
    }

    let ip = ip.unwrap_or_else(|| Ipv4Addr::new(127, 0, 0, 1).into());
    let port = port.unwrap_or(8787);
    let local_protocol = local_protocol.unwrap_or(Protocol::Http);
    let upstream_protocol = upstream_protocol.unwrap_or(Protocol::Https);

    let deployments = manifest.get_deployments(cli_params.environment.as_deref())?;
    let target = manifest.get_target(cli_params.environment.as_deref(), true)?;
    let user = GlobalUser::new().ok();

    let server_config = commands::dev::ServerConfig::new(host, ip, port, upstream_protocol)?;

    commands::dev::dev(
        target,
        deployments,
        user,
        server_config,
        local_protocol,
        upstream_protocol,
        cli_params.verbose,
        inspect,
    )
}
