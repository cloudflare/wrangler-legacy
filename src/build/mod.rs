use crate::settings::toml::{Target, TargetType};
use crate::terminal::message;
use crate::wranglerjs;
use crate::{commands, install};

use std::path::PathBuf;
use std::process::Command;

pub fn build(target: &Target) -> Result<(), failure::Error> {
    let target_type = &target.target_type;
    match target_type {
        TargetType::JavaScript => {
            message::info("JavaScript project found. Skipping unnecessary build!")
        }
        TargetType::Rust => {
            let tool_name = "wasm-pack";
            let binary_path = install::install(tool_name, "rustwasm")?.binary(tool_name)?;
            let args = ["build", "--target", "no-modules"];

            let command = command(&args, &binary_path);
            let command_name = format!("{:?}", command);

            commands::run(command, &command_name)?;
        }
        TargetType::Webpack => {
            wranglerjs::run_build(target)?;
        }
    }

    Ok(())
}

pub fn command(args: &[&str], binary_path: &PathBuf) -> Command {
    message::working("Compiling your project to WebAssembly...");

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
