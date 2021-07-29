use super::Cli;
use crate::build_target;
use crate::settings::toml::Manifest;
use crate::terminal::message::{Message, StdOut};

use anyhow::Result;

pub fn build(cli_params: &Cli) -> Result<()> {
    log::info!("Getting project settings");
    let manifest = Manifest::new(&cli_params.config)?;
    let target = manifest.get_target(cli_params.environment.as_deref(), false)?;
    build_target(&target).map(|msg| StdOut::success(&msg))
}
