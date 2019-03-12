use std::process::Command;

pub fn build() -> Result<(), failure::Error> {
    let build_wasm = "wasm-pack build --target no-modules";

    let _output = if cfg!(target_os = "windows") {
        Command::new("cmd").args(&["/C", build_wasm]).output()?
    } else {
        Command::new("sh").arg("-c").arg(build_wasm).output()?
    };
    Ok(())
}
