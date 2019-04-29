use crate::wranglerjs;
use binary_install::Cache;
use std::process::Command;

pub fn build(cache: &Cache) -> Result<(), failure::Error> {
    if !wranglerjs::is_installed() {
        println!("missing deps; installing...");
        wranglerjs::install();
    }

    wranglerjs::run_build()
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
