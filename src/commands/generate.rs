use crate::{commands, install};
use binary_install::Cache;
use std::process::Command;

pub fn generate(name: &str, template: &str, cache: &Cache) -> Result<(), failure::Error> {
    let tool_name = "cargo-generate";
    let binary_path = install::install(tool_name, "ashleygwilliams", cache)?.binary(tool_name)?;

    let worker_init = format!(
        "{} generate --git {} --name {}",
        binary_path.to_string_lossy(),
        template,
        name
    );
    commands::run(command(&worker_init, name), &worker_init)?;
    Ok(())
}

fn command(cmd: &str, name: &str) -> Command {
    println!(
        "🐑 Generating a new rustwasm worker project with name '{}'...",
        name
    );

    if cfg!(target_os = "windows") {
        let mut c = Command::new("cmd");
        c.arg(cmd);
        c.args(&["/C", cmd]);
        c
    } else {
        let mut c = Command::new("sh");
        c.arg("-c");
        c.arg(cmd);
        c
    }
}
