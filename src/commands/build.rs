use crate::{commands, install};
use binary_install::Cache;
use std::process::Command;

pub fn build(cache: &Cache) -> Result<(), failure::Error> {
    let tool_name = "wasm-pack";
    let binary_path = install::install(tool_name, "rustwasm", cache)?.binary(tool_name)?;
    let build_wasm = format!(
        "{} build --target no-modules",
        binary_path.to_string_lossy()
    );
    commands::run(command(&build_wasm), &build_wasm)?;
    Ok(())
}

fn command(cmd: &str) -> Command {
    println!("🌀 Compiling your project to WebAssembly...");

    if cfg!(target_os = "windows") {
        let mut c = Command::new("cmd");
        c.args(&["/C", cmd]);
        c
    } else {
        let mut c = Command::new("sh");
        c.arg("-c").arg(cmd);
        c
    }
}
