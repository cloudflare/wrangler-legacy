use super::Cli;
use crate::commands;
use crate::preview::{HttpMethod, PreviewOpt};
use crate::settings::{global_user::GlobalUser, toml::Manifest};

use anyhow::{ensure, Result};
use url::Url;

pub fn preview(
    method: HttpMethod,
    url: Url,
    body: Option<String>,
    watch: bool,
    headless: bool,
    cli_params: &Cli,
) -> Result<()> {
    log::info!("Getting project settings");
    let manifest = Manifest::new(&cli_params.config)?;
    let target = manifest.get_target(cli_params.environment.as_deref(), true)?;

    // the preview command can be called with or without a Global User having been config'd
    // so we convert this Result into an Option
    let user = GlobalUser::new().ok();

    // Validate the URL scheme
    ensure!(
        matches!(url.scheme(), "http" | "https"),
        "Invalid URL scheme (use either \"https\" or \"http\")"
    );

    let options = PreviewOpt {
        method,
        url,
        body,
        livereload: watch,
        headless,
    };

    commands::preview(target, user, options, cli_params.verbose)
}
