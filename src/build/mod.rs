use crate::settings::toml::{Target, TargetType};
use crate::terminal::message::{Message, StdOut};
use crate::terminal::styles;
use crate::wranglerjs;
use crate::{commands, install};

use std::path::PathBuf;
use std::process::Command;

mod check;

// Internal build logic, called by both `build` and `publish`
// TODO: return a struct containing optional build info and construct output at command layer
pub fn build_target(target: &Target) -> Result<String, failure::Error> {
    let target_type = &target.target_type;
    match target_type {
        TargetType::JavaScript => {
            let msg = "JavaScript project found. Skipping unnecessary build!".to_string();
            Ok(msg)
        }
        TargetType::Rust => {
            let _ = which::which("rustc").map_err(|e| {
                failure::format_err!(
                    "'rustc' not found: {}. Installation documentation can be found here: {}",
                    e,
                    styles::url("https://www.rust-lang.org/tools/install")
                )
            })?;

            let binary_path = install::install_wasm_pack()?;
            let args = ["build", "--target", "no-modules"];

            let command = command(&args, &binary_path);
            let command_name = format!("{:?}", command);

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

        TargetType::Bundler => match &target.bundle_config {
            None => Err(failure::err_msg("Please specify bundler options!")),
            Some(config) => {
                if config.build_command().spawn()?.wait()?.success() {
                    check::check_output_dir(config.output_dir()?)
                } else {
                    Err(failure::format_err!(
                        "Command `{:?}` failed!",
                        config.build_command()
                    ))
                }
            }
        },
    }
}

pub fn command(args: &[&str], binary_path: &PathBuf) -> Command {
    StdOut::working("Compiling your project to WebAssembly...");

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
