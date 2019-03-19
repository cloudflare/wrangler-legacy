use std::process::Command;

pub fn generate(name: &str) -> Result<(), failure::Error> {
    let worker_init = format!(
        "cargo generate --git https://github.com/cloudflare/rustwasm-worker-template --name {}",
        name
    );

    let _output = if cfg!(target_os = "windows") {
        Command::new("cmd").args(&["/C", &worker_init]).output()?
    } else {
        Command::new("sh").arg("-c").arg(&worker_init).output()?
    };
    Ok(())
}
