mod edge;
mod gcs;
mod server_config;
mod socket;

use server_config::ServerConfig;

use crate::settings::global_user::GlobalUser;
use crate::settings::toml::Target;
use crate::terminal::message_box;

pub fn dev(
    target: Target,
    user: Option<GlobalUser>,
    host: Option<&str>,
    port: Option<&str>,
    ip: Option<&str>,
    verbose: bool,
) -> Result<(), failure::Error> {
    message_box::dev_alpha_warning();
    let server_config = ServerConfig::new(host, ip, port)?;
    match user {
        Some(user) => edge::dev(target, user, server_config, verbose),
        None => gcs::dev(target, server_config, verbose),
    }
}
