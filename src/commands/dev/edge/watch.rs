use std::sync::{mpsc, Arc, Mutex};

use crate::commands::dev::edge::setup;
use crate::deploy::DeployTarget;
use crate::settings::global_user::GlobalUser;
use crate::settings::toml::Target;
use crate::watch::watch_and_build;

use anyhow::Result;

pub fn watch_for_changes(
    target: Target,
    deploy_target: &DeployTarget,
    user: &GlobalUser,
    preview_token: Arc<Mutex<String>>,
    session_token: String,
    verbose: bool,
) -> Result<()> {
    let (sender, receiver) = mpsc::channel();
    watch_and_build(&target, Some(sender))?;

    while receiver.recv().is_ok() {
        let user = user.clone();
        let target = target.clone();
        let deploy_target = deploy_target.clone();
        let session_token = session_token.clone();
        let mut target = target;

        // acquire the lock so incoming requests are halted
        // until the new script is ready for them
        let mut preview_token = preview_token.lock().unwrap();

        // while holding the lock, assign a new preview id
        //
        // this allows the server to route subsequent requests
        // to the proper script
        *preview_token = setup::upload(&mut target, &deploy_target, &user, session_token, verbose)?;
    }

    Ok(())
}
