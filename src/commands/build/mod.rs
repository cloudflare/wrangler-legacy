pub mod wranglerjs;

use crate::commands::publish::Package;
use crate::settings::project::{Project, ProjectType};
use crate::{commands, install};
use std::path::PathBuf;
use std::process::Command;

use crate::terminal::message;

use notify::{watcher, RecursiveMode, Watcher};
use std::sync::mpsc::{channel, Sender};
use std::thread;
use std::time::Duration;

use std::env;

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
            let package = Package::new("./")?;
            let entry = package.main()?;
            thread::spawn(move || {
                let (watcher_tx, watcher_rx) = channel();
                let mut watcher = watcher(watcher_tx, Duration::from_secs(1)).unwrap();

                watcher.watch(&entry, RecursiveMode::Recursive).unwrap();
                message::info(&format!("watching {:?}", &entry));

                loop {
                    match watcher_rx.recv() {
                        Ok(_) => {
                            message::working("Detected changes...");
                            if let Some(tx) = tx.clone() {
                                let _ = tx.send(());
                            }
                        },
                        Err(_) => panic!("Something went wrong while watching.")
                    }
                }
            });
        }
        ProjectType::Rust => {
            let tool_name = "wasm-pack";
            let binary_path = install::install(tool_name, "rustwasm")?.binary(tool_name)?;
            let args = ["build", "--target", "no-modules"];

            thread::spawn(move || {
                let (watcher_tx, watcher_rx) = channel();
                let mut watcher = watcher(watcher_tx, Duration::from_secs(1)).unwrap();

                let mut path = env::current_dir().expect("current dir");
                path.push("src");
 
                watcher.watch(&path, RecursiveMode::Recursive).unwrap();
                message::info(&format!("watching {:?}", &path));

                loop {
                    match watcher_rx.recv() {
                        Ok(_) => {
                            message::working("Detected changes...");
                            let command = command(&args, &binary_path);
                            let command_name = format!("{:?}", command);
                            if let Ok(_) = commands::run(command, &command_name) {
                                if let Some(tx) = tx.clone() {
                                    let _ = tx.send(());
                                }
                            }
                        },
                        Err(_) => panic!("Something went wrong while watching.")
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
