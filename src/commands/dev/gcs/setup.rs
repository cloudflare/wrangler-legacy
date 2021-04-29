use crate::commands::dev::ServerConfig;
use crate::preview::upload;
use crate::settings::global_user::GlobalUser;
use crate::settings::toml::Target;

use anyhow::Result;
use uuid::Uuid;

/// generate a unique uuid that lasts the entirety of the
/// `wrangler dev` session
pub(super) fn get_session_id() -> Result<String> {
    Ok(Uuid::new_v4().to_simple().to_string())
}

/// upload the script to the Workers API, and combine its response
/// with the session id to get the preview ID
///
/// this is used when sending requests to the Workers Runtime
/// so it executes the correct Worker
pub fn get_preview_id(
    mut target: Target,
    user: Option<GlobalUser>,
    server_config: &ServerConfig,
    session_id: &str,
    verbose: bool,
) -> Result<String> {
    // setting sites_preview to `true` would print a message to the terminal
    // directing the user to open the browser to view the output
    // this message makes sense for `wrangler preview` but not `wrangler dev`
    let sites_preview = false;
    let script_id = upload(&mut target, user.as_ref(), sites_preview, verbose)?;
    Ok(format!(
        "{}{}{}{}",
        &script_id,
        session_id,
        server_config.host.is_https() as u8,
        server_config.host
    ))
}
