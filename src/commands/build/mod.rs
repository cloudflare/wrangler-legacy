pub mod wranglerjs;

use crate::commands::publish::Package;
use crate::settings::project::{Project, ProjectType};
use crate::{commands, install};
use std::path::PathBuf;
use std::process::Command;

use crate::terminal::message;

use notify::{watcher, DebouncedEvent, RecursiveMode, Watcher};
use std::sync::mpsc::{channel, Sender};
use std::thread;
use std::time::Duration;

pub fn build(project: &Project) -> Result<(), failure::Error> {
    let project_type = &project.project_type;
    match project_type {
        ProjectType::JavaScript => {
            message::info("JavaScript project found. Skipping unnecessary build!")
        }
        ProjectType::Rust => {
            let tool_name = "wasm-pack";
            let binary_path = install::install(tool_name, "rustwasm")?.binary(tool_name)?;
            let args = ["build", "--target", "no-modules"];

            let command = command(&args, &binary_path);
            let command_name = format!("{:?}", command);

            commands::run(command, &command_name)?;
        }
        ProjectType::Webpack => {
            wranglerjs::run_build(project)?;
        }
    }

    Ok(())
}

pub fn build_and_watch(project: &Project, tx: Option<Sender<()>>) -> Result<(), failure::Error> {
    let project_type = &project.project_type;
    match project_type {
        ProjectType::JavaScript => {
            let (watcher_tx, watcher_rx) = channel();
            let mut watcher = watcher(watcher_tx, Duration::from_secs(1))?;

            let package = Package::new("./")?;
            watcher.watch(package.main()?, RecursiveMode::Recursive)?;

            thread::spawn(move || loop {
                if let Ok(DebouncedEvent::Write(_path)) = watcher_rx.recv() {
                    if let Some(tx) = tx.clone() {
                        tx.send(());
                    }
                }
            });
        }
        ProjectType::Rust => {
            let tool_name = "wasm-pack";
            let binary_path = install::install(tool_name, "rustwasm")?.binary(tool_name)?;
            let args = ["build", "--target", "no-modules"];


            let (watcher_tx, watcher_rx) = channel();
            let mut watcher = watcher(watcher_tx, Duration::from_secs(1))?;

            watcher.watch("./src", RecursiveMode::Recursive)?;

            thread::spawn(move || loop {
                if let Ok(DebouncedEvent::Write(_path)) = watcher_rx.recv() {
                    let command = command(&args, &binary_path);
                    let command_name = format!("{:?}", command);
                    if let Ok(_) = commands::run(command, &command_name) {
                        if let Some(tx) = tx.clone() {
                            tx.send(());
                        }
                    }
                }
            });
        }
        ProjectType::Webpack => {
            wranglerjs::run_build_and_watch(project, tx)?;
        }
    }

    Ok(())
}

fn command(args: &[&str], binary_path: &PathBuf) -> Command {
    message::working("Compiling your project to WebAssembly...");

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
