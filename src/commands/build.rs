use crate::user::settings::ProjectType;
use crate::wranglerjs;
use crate::{commands, install};
use binary_install::Cache;
use std::path::PathBuf;
use std::process::Command;

pub fn build(cache: &Cache, project_type: &ProjectType) -> Result<(), failure::Error> {
    match project_type {
        _ => wranglerjs::run_build(),
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
