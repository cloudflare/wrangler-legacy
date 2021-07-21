use super::Cli;
use crate::commands;
use crate::settings::{global_user::GlobalUser, toml::Manifest};

use anyhow::Result;

pub fn tail(cli_params: &Cli) -> Result<()> {
    let manifest = Manifest::new(&cli_params.config)?;
    let target = manifest.get_target(cli_params.environment.as_deref(), false)?;
    let user = GlobalUser::new()?;
    commands::tail::start(target, user)
}
