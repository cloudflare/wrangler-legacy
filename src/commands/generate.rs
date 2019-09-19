use crate::settings::target::{Manifest, TargetType};
use crate::{commands, install};
use std::path::PathBuf;
use std::process::Command;

use crate::terminal::{emoji, message};

pub fn generate(
    name: &str,
    template: &str,
    target_type: Option<TargetType>,
    site: bool,
) -> Result<(), failure::Error> {
    let target_type = target_type.unwrap_or_else(|| get_target_type(template));
    if site {
        run_generate(name, template, &target_type)?;
        let config_path = PathBuf::from("./").join(&name);
        Manifest::generate(name.to_string(), target_type, config_path, site)?;
        Ok(())
    } else {
        run_generate(name, template, &target_type)?;
        let config_path = PathBuf::from("./").join(&name);
        Manifest::generate(name.to_string(), target_type, config_path, site)?;
        Ok(())
    }
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
