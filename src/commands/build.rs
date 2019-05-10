use binary_install::Cache;
use std::process::Command;

use crate::wranglerjs;

pub fn build(cache: &Cache) -> Result<(), failure::Error> {
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
