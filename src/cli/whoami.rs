use crate::commands;
use crate::login::check_update_oauth_token;
use crate::settings::global_user::GlobalUser;

use anyhow::Result;

pub fn whoami() -> Result<()> {
    log::info!("Getting User settings");
    let mut user = GlobalUser::new()?;

    // Check if oauth token is expired
    let _res = check_update_oauth_token(&mut user).expect("Failed to refresh access token");

    commands::whoami(&mut user)
}
