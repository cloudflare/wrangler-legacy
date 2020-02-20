mod edge;
mod gcs;
mod server_config;
mod socket;
mod utils;

use server_config::ServerConfig;

use crate::settings::global_user::GlobalUser;
use crate::settings::toml::{DeployConfig, Target};
use crate::terminal::message_box;

pub fn dev(
    target: Target,
    deploy_config: DeployConfig,
    user: Option<GlobalUser>,
    host: Option<&str>,
    port: Option<&str>,
    ip: Option<&str>,
    verbose: bool,
) -> Result<(), failure::Error> {
    message_box::dev_alpha_warning();
    let server_config = ServerConfig::new(host, ip, port)?;
    match user {
        Some(user) => edge::dev(target, deploy_config, user, server_config, verbose),
        None => gcs::dev(target, server_config, verbose),
    }
}
