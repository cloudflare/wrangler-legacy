use crate::settings::project::{Project, ProjectType};
use crate::{commands, install};
use std::path::PathBuf;
use std::process::Command;

use crate::terminal::{emoji, message};

pub fn generate(
    name: &str,
    template: &str,
    project_type: ProjectType,
) -> Result<(), failure::Error> {
    let tool_name = "cargo-generate";
    let binary_path = install::install(tool_name, "ashleygwilliams")?.binary(tool_name)?;

    let args = ["generate", "--git", template, "--name", name, "--force"];

    let command = command(name, binary_path, &args, &project_type);
    let command_name = format!("{:?}", command);

    commands::run(command, &command_name)?;
    Project::generate(name.to_string(), project_type, false)?;
    Ok(())
}

fn command(name: &str, binary_path: PathBuf, args: &[&str], project_type: &ProjectType) -> Command {
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
