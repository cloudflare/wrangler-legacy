use uuid::Uuid;

use crate::commands::dev::ServerConfig;
use crate::commands::preview::upload;
use crate::settings::global_user::GlobalUser;
use crate::settings::toml::Target;

pub(super) fn get_preview_id(
    mut target: Target,
    user: Option<GlobalUser>,
    server_config: &ServerConfig,
    session_id: &str,
    verbose: bool,
) -> Result<String, failure::Error> {
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

pub(super) fn get_session_id() -> Result<String, failure::Error> {
    Ok(Uuid::new_v4().to_simple().to_string())
}
