use crate::{commands, install};
use binary_install::Cache;
use std::path::PathBuf;
use std::process::Command;

pub fn build(cache: &Cache) -> Result<(), failure::Error> {
    let tool_name = "wasm-pack";
    let binary_path = install::install(tool_name, "rustwasm", cache)?.binary(tool_name)?;

    let args = ["build", "--target", "no-modules"];

    let command = command(&args, binary_path);
    let command_name = format!("{:?}", command);

    commands::run(command, &command_name)?;
    Ok(())
}

fn command(args: &[&str], binary_path: PathBuf) -> Command {
    println!("🌀 Compiling your project to WebAssembly...");

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
