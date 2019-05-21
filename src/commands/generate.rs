use crate::user::settings::{ProjectSettings, ProjectType};
use crate::{commands, install};
use binary_install::Cache;
use std::path::PathBuf;
use std::process::Command;

pub fn generate(name: &str, template: &str, cache: &Cache) -> Result<(), failure::Error> {
    let tool_name = "cargo-generate";
    let binary_path = install::install(tool_name, "ashleygwilliams", cache)?.binary(tool_name)?;

    let args = ["generate", "--git", template, "--name", name];

    let project_type = project_type(template);
    let command = command(name, binary_path, &args, &project_type);
    let command_name = format!("{:?}", command);

    commands::run(command, &command_name)?;
    ProjectSettings::generate(name.to_string(), project_type)?;
    Ok(())
}

fn command(name: &str, binary_path: PathBuf, args: &[&str], project_type: &ProjectType) -> Command {
    println!(
        "🐑 Generating a new {} worker project with name '{}'...",
        project_type, name
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
    ProjectType::Webpack
}
