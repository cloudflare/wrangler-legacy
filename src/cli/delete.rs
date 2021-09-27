use super::Cli;
use crate::commands;
use crate::settings::{global_user::GlobalUser, toml::Manifest};

pub fn delete(force: bool, cli_params: &Cli) -> Result<(), anyhow::Error> {
    // Get user info
    let user = GlobalUser::new()?;

    // Get project info
    let manifest = Manifest::new(&cli_params.config)?;

    let account_id = manifest.account_id.load()?;
    let script_name = manifest.worker_name(None);

    commands::delete::delete_script(&user, force, account_id, &script_name)
}
