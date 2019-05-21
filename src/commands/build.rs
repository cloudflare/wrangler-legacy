use crate::user::settings::ProjectType;
use crate::wranglerjs;
use crate::{commands, install};
use binary_install::Cache;
use std::path::PathBuf;
use std::process::Command;

pub fn build(cache: &Cache, project_type: &ProjectType) -> Result<(), failure::Error> {
    match project_type {
        ProjectType::JavaScript => {
            println!("⚠️ JavaScript project found. Skipping unecessary build!")
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
            if !wranglerjs::is_installed() {
                println!("missing deps; installing...");
                wranglerjs::install().expect("could not install wranglerjs");
            }

            let wranglerjs_output = wranglerjs::run_build().expect("could not run wranglerjs");
            let bundle = wranglerjs::Bundle::new();
            let out = wranglerjs_output.compiler_output();

            bundle
                .write(wranglerjs_output)
                .expect("could not write bundle to disk");

            println!("{}", out);
        }
    }

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
