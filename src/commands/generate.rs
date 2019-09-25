use std::path::PathBuf;
use std::process::Command;

use crate::commands::validate_worker_name;
use crate::settings::target::{Manifest, Site, TargetType};
use crate::terminal::{emoji, message};
use crate::{commands, install};
use crate::settings::ignore;

pub fn generate(
    name: &str,
    template: &str,
    target_type: Option<TargetType>,
    site: bool,
) -> Result<(), failure::Error> {
    validate_worker_name(name)?;

    let target_type = target_type.unwrap_or_else(|| get_target_type(template));
    run_generate(name, template, &target_type)?;
    let config_path = PathBuf::from("./").join(&name);
    // TODO: this is tightly coupled to our site template. Need to remove once
    // we refine our generate logic.
    let generated_site = if site {
        Some(Site::new("./public"))
    } else {
        None
    };
    Manifest::generate(name.to_string(), target_type, config_path, generated_site)?;

    // Writes .wranglerignore file.
    ignore::write_default_wranglerignore()?;

    Ok(())
}

pub fn run_generate(
    name: &str,
    template: &str,
    target_type: &TargetType,
) -> Result<(), failure::Error> {
    let tool_name = "cargo-generate";
    let binary_path = install::install(tool_name, "ashleygwilliams")?.binary(tool_name)?;

    let args = ["generate", "--git", template, "--name", name, "--force"];

    let command = command(name, binary_path, &args, &target_type);
    let command_name = format!("{:?}", command);

    commands::run(command, &command_name)?;
    Ok(())
}

fn command(name: &str, binary_path: PathBuf, args: &[&str], project_type: &TargetType) -> Command {
    let msg = format!(
        "{} Generating a new {} worker project with name '{}'...",
        emoji::SHEEP,
        project_type,
        name
    );

    message::working(&msg);

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

fn get_target_type(template: &str) -> TargetType {
    if template.contains("rust") {
        return TargetType::Rust;
    }
    TargetType::default()
}
