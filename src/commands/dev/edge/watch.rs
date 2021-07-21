use std::sync::{mpsc, Arc, Mutex};

use crate::commands::dev::edge::setup;
use crate::deploy::DeployTarget;
use crate::settings::global_user::GlobalUser;
use crate::settings::toml::Target;
use crate::terminal::message::{Message, StdOut};
use crate::watch::watch_and_build;

use anyhow::Result;

use super::setup::Session;

pub fn watch_for_changes(
    target: Target,
    deploy_target: &DeployTarget,
    user: &GlobalUser,
    session: Arc<Mutex<Session>>,
    preview_token: Arc<Mutex<String>>,
    verbose: bool,
) -> Result<()> {
    let (sender, receiver) = mpsc::channel();
    watch_and_build(&target, Some(sender))?;

    loop {
        if let Err(e) = receiver.recv() {
            panic!("The channel for file changes was disconnected: {}", e);
        }
        let user = user.clone();
        let target = target.clone();
        let deploy_target = deploy_target.clone();
        let mut target = target;

        // acquire the lock so incoming requests are halted
        // until the new script is ready for them
        let mut preview_token = preview_token.lock().unwrap();

        // while holding the lock, assign a new preview id
        //
        // this allows the server to route subsequent requests
        // to the proper script
        let uploaded = setup::upload(&mut target, &deploy_target, &user, session.clone(), verbose);
        *preview_token = match uploaded {
            Ok(token) => token,
            Err(_) => {
                {
                    StdOut::info("Starting a new session because the existing token has expired");
                    let mut session = session.lock().unwrap();
                    *session = Session::new(&target, &user, &deploy_target)?;
                }
                setup::upload(&mut target, &deploy_target, &user, session.clone(), verbose)
                    .expect("Failed to upload the changes after starting a new session")
            }
        };
    }
}
