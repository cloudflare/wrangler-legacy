mod edge;
mod gcs;
mod server_config;
mod socket;
mod utils;

use server_config::ServerConfig;

use crate::build;
use crate::settings::global_user::GlobalUser;
use crate::settings::toml::{DeployConfig, Target};
use crate::terminal::{message, styles};

/// `wrangler dev` starts a server on a dev machine that routes incoming HTTP requests
/// to a Cloudflare Workers runtime and returns HTTP responses
pub fn dev(
    target: Target,
    deploy_config: DeployConfig,
    user: Option<GlobalUser>,
    host: Option<&str>,
    port: Option<&str>,
    ip: Option<&str>,
    verbose: bool,
) -> Result<(), failure::Error> {
    let server_config = ServerConfig::new(host, ip, port)?;

    // we can remove this once the feature has stabilized
    print_alpha_warning_message();

    // before serving requests we must first build the Worker
    build(&target)?;

    match user {
        // authenticated users connect to the edge
        Some(user) => edge::dev(target, deploy_config, user, server_config),

        // unauthenticated users connect to gcs
        None => gcs::dev(target, server_config, verbose),
    }
}

fn print_alpha_warning_message() {
    let wrangler_dev_msg = styles::highlight("`wrangler dev`");
    let feedback_url = styles::url("https://github.com/cloudflare/wrangler/issues/1047");
    message::billboard(&format!("{0} is currently unstable and there are likely to be breaking changes!\nFor this reason, we cannot yet recommend using {0} for integration testing.\n\nPlease submit any feedback here: {1}", wrangler_dev_msg, feedback_url));
}
