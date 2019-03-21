mod cargo_generate;

use crate::commands;
use binary_install::Cache;
use std::process::Command;

pub fn generate(name: &str, cache: Cache) -> Result<(), failure::Error> {
    let binary_path = cargo_generate::install(&cache)?.binary("cargo-generate")?;

    let worker_init = format!(
        "{} generate --git https://github.com/cloudflare/rustwasm-worker-template --name {}",
        binary_path.to_string_lossy(),
        name
    );
    commands::run(command(&worker_init, name), &worker_init)?;
    Ok(())
}

pub fn command(cmd: &str, name: &str) -> Command {
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
