mod edge;
mod gcs;
mod server_config;
mod socket;
mod utils;

use console::style;
use server_config::ServerConfig;

use crate::settings::global_user::GlobalUser;
use crate::settings::toml::{DeployConfig, Target};
use crate::terminal::message;

pub fn dev(
    target: Target,
    deploy_config: DeployConfig,
    user: Option<GlobalUser>,
    host: Option<&str>,
    port: Option<&str>,
    ip: Option<&str>,
    verbose: bool,
) -> Result<(), failure::Error> {
    let wrangler_dev_msg = style("`wrangler dev`").yellow().bold();
    let feedback_url = style("https://github.com/cloudflare/wrangler/issues/1047")
        .blue()
        .bold();
    message::billboard(&format!("{0} is currently unstable and there are likely to be breaking changes!\nFor this reason, we cannot yet recommend using {0} for integration testing.\n\nPlease submit any feedback here: {1}", wrangler_dev_msg, feedback_url));

    let server_config = ServerConfig::new(host, ip, port)?;
    match user {
        Some(user) => edge::dev(target, deploy_config, user, server_config),
        None => gcs::dev(target, server_config, verbose),
    }
}
