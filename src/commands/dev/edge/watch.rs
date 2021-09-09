use std::sync::mpsc::Sender;
use std::sync::{mpsc, Arc, Mutex};

use crate::commands::dev::edge::setup;
use crate::deploy::DeployTarget;
use crate::settings::global_user::GlobalUser;
use crate::settings::toml::Target;
use crate::terminal::message::{Message, StdOut};
use crate::watch::watch_and_build;

use anyhow::Result;

pub fn watch_for_changes(
    target: &Target,
    deploy_target: &DeployTarget,
    user: &GlobalUser,
    preview_token: Arc<Mutex<String>>,
    session_token: String,
    verbose: bool,
    refresh_session_channel: Sender<Option<()>>,
) -> Result<()> {
    let (sender, receiver) = mpsc::channel();
    watch_and_build(target, Some(sender), Some(refresh_session_channel.clone()))?;

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
        let uploaded = setup::upload(&mut target, &deploy_target, &user, session_token, verbose);
        match uploaded {
            Ok(token) => {
                *preview_token = token;
            }
            Err(err) => {
                if let Some(err) = err.downcast_ref::<setup::BadRequestError>() {
                    // no need to refresh session here, bad request indicates problem with user code
                    StdOut::warn(&err.0);
                } else {
                    refresh_session_channel.send(Some(()))?;
                    break;
                }
            }
        }
    }

    Ok(())
}
