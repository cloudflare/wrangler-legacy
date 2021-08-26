use super::Cli;
use crate::commands;
use crate::login::check_update_oauth_token;
use crate::settings::{global_user::GlobalUser, toml::Manifest};

use anyhow::Result;

pub fn subdomain(name: Option<String>, cli_params: &Cli) -> Result<()> {
    log::info!("Getting project settings");
    let manifest = Manifest::new(&cli_params.config)?;
    let target = manifest.get_target(cli_params.environment.as_deref(), false)?;

    log::info!("Getting User settings");
    let mut user = GlobalUser::new()?;

    // Check if oauth token is expired
    let _res = check_update_oauth_token(&mut user).expect("Failed to refresh access token");

    if let Some(name) = name {
        commands::subdomain::set_subdomain(&name, &user, &target)
    } else {
        commands::subdomain::get_subdomain(&user, &target)
    }
}
