use std::path::Path;

use clap::ArgMatches;

use crate::build;
use crate::settings::toml::Manifest;

use super::DEFAULT_CONFIG_PATH;

pub fn run(matches: &ArgMatches) -> Result<(), failure::Error> {
    log::info!("Getting project settings");
    let config_path = Path::new(DEFAULT_CONFIG_PATH);
    let manifest = Manifest::new(&config_path)?;
    let env = matches.value_of("env");
    let target = &manifest.get_target(env, false)?;

    let env = env.map(|s| s.to_string());
    build(&target, env)
}
