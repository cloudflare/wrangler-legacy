use super::Cli;
use crate::commands;
use crate::settings::{global_user::GlobalUser, toml::Manifest};

use anyhow::Result;

pub fn subdomain(name: Option<String>, cli_params: &Cli) -> Result<()> {
    log::info!("Getting project settings");
    let manifest = Manifest::new(&cli_params.config)?;
    let target = manifest.get_target(cli_params.environment.as_deref(), false)?;

    log::info!("Getting User settings");
    let user = GlobalUser::new()?;

    if let Some(name) = name {
        commands::subdomain::set_subdomain(&name, &user, &target)
    } else {
        commands::subdomain::get_subdomain(&user, &target)
    }
}
