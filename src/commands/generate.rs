use crate::user::settings::ProjectSettings;
use crate::{commands, install};
use binary_install::Cache;
use std::process::Command;

pub fn generate(name: &str, template: &str, cache: &Cache) -> Result<(), failure::Error> {
    let tool_name = "cargo-generate";
    let binary_path = install::install(tool_name, "ashleygwilliams", cache)?.binary(tool_name)?;

    let args = [
        &*binary_path.to_string_lossy(),
        "generate",
        "--git",
        template,
        "--name",
        name,
    ];

    let command = command(name, &args);
    let command_name = format!("{:?}", command);

    commands::run(command, &command_name)?;
    ProjectSettings::generate(name.to_string())?;
    Ok(())
}

fn command(name: &str, args: &[&str]) -> Command {
    println!(
        "🐑 Generating a new rustwasm worker project with name '{}'...",
        name
    );

    let mut c = if cfg!(target_os = "windows") {
        let mut c = Command::new("cmd");
        c.arg("/C");
        c
    } else {
        let mut c = Command::new("sh");
        c.arg("-c");
        c
    };

    c.args(args);

    c
}
