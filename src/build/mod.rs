use crate::settings::toml::{Target, TargetType};
use crate::terminal::message::{Message, StdErr};
use crate::terminal::styles;
use crate::wranglerjs;
use crate::{commands, install};

use std::path::Path;
use std::process::Command;

use anyhow::{anyhow, Result};

// Internal build logic, called by both `build` and `publish`
// TODO: return a struct containing optional build info and construct output at command layer
pub fn build_target(target: &Target) -> Result<String> {
    let target_type = &target.target_type;
    match target_type {
        TargetType::JavaScript => match &target.build {
            None => {
                let msg = "Basic JavaScript project found. Skipping unnecessary build!".to_string();
                Ok(msg)
            }
            Some(config) => {
                if let Some((cmd_str, mut cmd)) = config.build_command() {
                    StdErr::working(format!("Running {}", cmd_str).as_ref());
                    let build_result = cmd.spawn()?.wait()?;
                    if build_result.success() {
                        Ok(String::from("Build completed successfully!"))
                    } else if let Some(code) = build_result.code() {
                        Err(anyhow!("Build failed! Status Code: {}", code))
                    } else {
                        Err(anyhow!("Build failed."))
                    }
                } else {
                    Ok(String::from("No build command specified, skipping build."))
                }
            }
        },
        TargetType::Rust => {
            let _ = which::which("rustc").map_err(|e| {
                anyhow!(
                    "'rustc' not found: {}. Installation documentation can be found here: {}",
                    e,
                    styles::url("https://www.rust-lang.org/tools/install")
                )
            })?;

            let binary_path = install::install_wasm_pack()?;
            let args = ["build", "--target", "no-modules"];

            let command = command(&args, &binary_path);
            let command_name = format!("{:?}", command);

            StdErr::working("Compiling your project to WebAssembly...");
            commands::run(command, &command_name)?;
            let msg = "Build succeeded".to_string();
            Ok(msg)
        }
        TargetType::Webpack => match wranglerjs::run_build(target) {
            Ok(output) => {
                let msg = format!(
                    "Built successfully, built project size is {}",
                    output.project_size()
                );
                Ok(msg)
            }
            Err(e) => Err(e),
        },
    }
}

pub fn command(args: &[&str], binary_path: &Path) -> Command {
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
