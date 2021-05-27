use super::Cli;
use crate::commands;
use crate::settings::{global_user::GlobalUser, toml::Manifest};

use anyhow::Result;

pub fn tail(
    format: String,
    tunnel_port: Option<u16>,
    metrics_port: Option<u16>,
    cli_params: &Cli,
) -> Result<()> {
    let manifest = Manifest::new(&cli_params.config)?;
    let target = manifest.get_target(cli_params.environment.as_deref(), false)?;
    let user = GlobalUser::new()?;

    commands::tail::start(
        &target,
        &user,
        format,
        tunnel_port,
        metrics_port,
        cli_params.verbose,
    )
}
