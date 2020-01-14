use std::mem;
use std::sync::{mpsc, Arc, Mutex};

use crate::commands;
use crate::commands::dev::get_preview_id;
use crate::commands::dev::server_config::ServerConfig;

use crate::settings::global_user::GlobalUser;
use crate::settings::toml::Target;

pub fn watch_for_changes(
    target: Target,
    user: Option<GlobalUser>,
    server_config: &ServerConfig,
    preview_id: Arc<Mutex<String>>,
    session_id: &str,
    verbose: bool,
) -> Result<(), failure::Error> {
    let (tx, rx) = mpsc::channel();
    commands::watch_and_build(&target, Some(tx))?;

    while let Ok(_) = rx.recv() {
        let user = user.clone();
        let target = target.clone();
        commands::build(&target)?;

        let mut p = preview_id.lock().unwrap();
        *p = get_preview_id(target, user, server_config, session_id, verbose)?;
        mem::drop(p);
    }

    Ok(())
}
