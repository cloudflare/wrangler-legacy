use crate::commands;
use crate::settings::global_user::GlobalUser;

use anyhow::Result;

pub fn whoami() -> Result<()> {
    log::info!("Getting User settings");
    commands::whoami(&GlobalUser::new()?)
}
