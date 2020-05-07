mod gcs;
mod server_config;
mod socket;
use server_config::ServerConfig;

use crate::commands;
use crate::settings::global_user::GlobalUser;
use crate::settings::toml::Target;
use crate::terminal::{message, styles};

/// `wrangler dev` starts a server on a dev machine that routes incoming HTTP requests
/// to a Cloudflare Workers runtime and returns HTTP responses
pub fn dev(
    target: Target,
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
    commands::build(&target)?;

    // eventually we will have two modes - edge and gcs
    // edge for authenticated users and gcs for unauthenticated
    // for now, always route to gcs
    gcs::dev(target, user, server_config, verbose)
}

fn print_alpha_warning_message() {
    let wrangler_dev_msg = styles::highlight("`wrangler dev`");
    let feedback_url = styles::url("https://github.com/cloudflare/wrangler/issues/1047");
    message::billboard(&format!("{0} is currently unstable and there are likely to be breaking changes!\nFor this reason, we cannot yet recommend using {0} for integration testing.\n\nPlease submit any feedback here: {1}", wrangler_dev_msg, feedback_url));
}
