use std::path::PathBuf;
use std::process::Command;

use crate::commands::validate_worker_name;
use crate::settings::toml::{Manifest, Site, TargetType};
use crate::{commands, install};

pub fn generate(
    name: &str,
    template: &str,
    target_type: Option<TargetType>,
    site: bool,
) -> Result<(), failure::Error> {
    validate_worker_name(name)?;

    log::info!("Generating a new worker project with name '{}'", name);
    run_generate(name, template)?;

    let config_path = PathBuf::from("./").join(&name);
    // TODO: this is tightly coupled to our site template. Need to remove once
    // we refine our generate logic.
    let generated_site = if site {
        Some(Site::new("./public"))
    } else {
        None
    };
    Manifest::generate(name.to_string(), target_type, &config_path, generated_site)?;

    Ok(())
}

pub fn run_generate(name: &str, template: &str) -> Result<(), failure::Error> {
    let tool_name = "cargo-generate";
    let tool_author = "ashleygwilliams";
    let is_binary = true;
    let version = install::get_latest_version(tool_name)?;
    let binary_path =
        install::install(tool_name, tool_author, is_binary, version)?.binary(tool_name)?;

    let args = ["generate", "--git", template, "--name", name, "--force"];

    let command = command(binary_path, &args);
    let command_name = format!("{:?}", command);

    commands::run(command, &command_name)
}

fn command(binary_path: PathBuf, args: &[&str]) -> Command {
    let mut c = if cfg!(target_os = "windows") {
        let mut c = Command::new("cmd");
        c.arg("/C");
        c.arg(binary_path);
        c
    } else {
        Command::new(binary_path)
    };

    c.args(args);
    c
}
