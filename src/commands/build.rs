use crate::commands;
use std::process::Command;

pub fn build() -> Result<(), failure::Error> {
    let build_wasm = "wasm-pack build --target no-modules";
    commands::run(command(build_wasm), build_wasm)?;
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
