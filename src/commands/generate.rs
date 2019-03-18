use std::process::Command;

pub fn generate() -> Result<(), failure::Error> {
    let worker_init =
        "cargo generate --git https://github.com/rustwasm/wasm-pack-template.git --name wasm-worker";

    let _output = if cfg!(target_os = "windows") {
        Command::new("cmd").args(&["/C", worker_init]).output()?
    } else {
        Command::new("sh").arg("-c").arg(worker_init).output()?
    };
    Ok(())
}
