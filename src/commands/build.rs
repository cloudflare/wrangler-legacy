use std::path::Path;

use clap::ArgMatches;

use super::DEFAULT_CONFIG_PATH;
use crate::build;
use crate::settings::toml::Manifest;
use crate::terminal;

pub fn run(matches: &ArgMatches) -> Result<(), failure::Error> {
    log::info!("Getting project settings");
    let config_file = matches.value_of("config").unwrap_or(DEFAULT_CONFIG_PATH);
    let config_path = Path::new(config_file);
    let manifest = Manifest::new(&config_path)?;
    let env = matches.value_of("env");
    let target = &manifest.get_target(env, false)?;
    if matches.is_present("output") {
        if matches.value_of("output") == Some("json") {
            terminal::message::set_output_type(terminal::message::OutputType::Json)
        } else {
            terminal::message::user_error("json is the only valid value for output flag");
        }
        let result = build(&target);
        let jsonoutput = terminal::message::PublishOutput {
            name: Some(target.name.clone()),
            success: match result {
                Ok(_) => Some(true),
                Err(_) => Some(false),
            },
            urls: Vec::new(),
        };
        terminal::message::jsonout(&jsonoutput);
        result
    } else {
        terminal::message::set_output_type(terminal::message::OutputType::Human);
        build(&target)
    }
}
