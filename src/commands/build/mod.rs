use crate::emoji;
use crate::settings::project::ProjectType;
use crate::{commands, install};
use binary_install::Cache;
use std::path::PathBuf;
use std::process::Command;

pub mod webpack;

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
            let wasm_pack_path =
                install::install("wasm-pack", "rustwasm", cache)?.binary("wasm-pack")?;

            webpack::run_npm_install().expect("could not run `npm install`");

            webpack::install().expect("could not install wranglerjs");

            let bundle = webpack::Bundle::new();
            let webpack_output =
                webpack::run_build(wasm_pack_path, &bundle).expect("could not run wranglerjs");

            bundle
                .write(webpack_output)
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
