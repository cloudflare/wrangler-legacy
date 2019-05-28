use crate::settings::project::ProjectType;
use crate::wranglerjs;
use crate::{commands, install};
use binary_install::Cache;
use std::env;
use std::path::PathBuf;
use std::process::Command;

use crate::emoji;

pub fn build(cache: &Cache, project_type: &ProjectType) -> Result<(), failure::Error> {
    match project_type {
        ProjectType::JavaScript => {
            println!("⚠️ JavaScript project found. Skipping unnecessary build!")
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
            for tool in vec!["node", "npm"] {
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

            bundle
                .write(wranglerjs_output)
                .expect("could not write bundle to disk");
        }
    }

    Ok(())
}

fn command(args: &[&str], binary_path: PathBuf) -> Command {
    println!("{} Compiling your project to WebAssembly...", emoji::SWIRL);

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
