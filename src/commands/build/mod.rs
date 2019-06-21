pub mod wranglerjs;

use crate::settings::project::ProjectType;
use crate::{commands, install};
use binary_install::Cache;
use std::env;
use std::path::PathBuf;
use std::process::Command;

use crate::terminal::message;

pub fn build(cache: &Cache, project_type: &ProjectType) -> Result<(), failure::Error> {
    match project_type {
        ProjectType::JavaScript => {
            message::info("JavaScript project found. Skipping unnecessary build!")
        }
        ProjectType::Rust => {
            let tool_name = "wasm-pack";
            let binary_path = install::install(tool_name, "rustwasm", cache)?.binary(tool_name)?;
            let args = ["build", "--target", "no-modules"];

            let command = command(&args, binary_path);
            let command_name = format!("{:?}", command);

            commands::run(command, &command_name)?;
        }
        ProjectType::Webpack => {
            for tool in &["node", "npm"] {
                wranglerjs::env_dep_installed(tool)?;
            }

            let wasm_pack_path =
                install::install("wasm-pack", "rustwasm", cache)?.binary("wasm-pack")?;
            let wranglerjs_path = wranglerjs::install(cache).expect("could not install wranglerjs");

            let current_dir = env::current_dir()?;
            wranglerjs::run_npm_install(current_dir).expect("could not run `npm install`");

            let bundle = wranglerjs::Bundle::new();
            let wranglerjs_output = wranglerjs::run_build(wranglerjs_path, wasm_pack_path, &bundle)
                .expect("could not run wranglerjs");

            if wranglerjs_output.has_errors() {
                message::user_error(&format!("{}", wranglerjs_output.get_errors()));
                failure::bail!("Webpack returned an error");
            }

            bundle
                .write(&wranglerjs_output)
                .expect("could not write bundle to disk");

            let mut msg = format!(
                "Built successfully, script size is {}",
                wranglerjs_output.script_size()
            );
            if bundle.has_wasm() {
                msg = format!("{} and Wasm size is {}", msg, wranglerjs_output.wasm_size());
            }
            message::success(&msg);
        }
    }

    Ok(())
}

fn command(args: &[&str], binary_path: PathBuf) -> Command {
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
