use crate::settings::project::{Project, ProjectType};
use crate::{commands, install};
use binary_install::Cache;
use std::path::PathBuf;
use std::process::Command;

use crate::emoji;

pub fn generate(
    name: &str,
    template: &str,
    pt: Option<ProjectType>,
    cache: &Cache,
) -> Result<(), failure::Error> {
    let tool_name = "cargo-generate";
    let binary_path = install::install(tool_name, "ashleygwilliams", cache)?.binary(tool_name)?;

    let args = ["generate", "--git", template, "--name", name, "--force"];

    let pt = pt.unwrap_or_else(|| project_type(template));
    let command = command(name, binary_path, &args, &pt);
    let command_name = format!("{:?}", command);

    commands::run(command, &command_name)?;
    Project::generate(name.to_string(), pt, false)?;
    Ok(())
}

fn command(name: &str, binary_path: PathBuf, args: &[&str], project_type: &ProjectType) -> Command {
    println!(
        "{} Generating a new {} worker project with name '{}'...",
        emoji::SHEEP,
        project_type,
        name
    );

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

fn project_type(template: &str) -> ProjectType {
    if template.contains("rust") {
        return ProjectType::Rust;
    }
    ProjectType::default()
}
