use std::path::Path;

use clap::ArgMatches;

use super::DEFAULT_CONFIG_PATH;
use crate::build::build_target;
use crate::settings::toml::Manifest;
use crate::terminal::message::{Message, StdOut};

pub fn build(matches: &ArgMatches) -> Result<(), failure::Error> {
    log::info!("Getting project settings");
    let config_file = matches.value_of("config").unwrap_or(DEFAULT_CONFIG_PATH);
    let config_path = Path::new(config_file);
    let manifest = Manifest::new(&config_path)?;
    let env = matches.value_of("env");
    let target = &manifest.get_target(env, false)?;
    let build_result = build_target(&target);
    match build_result {
        Ok(msg) => {
            StdOut::success(&msg);
            Ok(())
        }
        Err(e) => Err(e),
    }
}
