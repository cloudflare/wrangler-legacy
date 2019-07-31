use crate::commands::build::{command, wranglerjs};
use crate::commands::publish::Package;
use crate::settings::project::{Project, ProjectType};
use crate::terminal::message;
use crate::{commands, install};

use notify::{watcher, RecursiveMode, Watcher};
use std::env;
use std::sync::mpsc::{channel, Sender};
use std::thread;
use std::time::Duration;

/// watch a project for changes and re-build it when necessary,
/// outputting a build event to tx.
pub fn watch_and_build(project: &Project, tx: Option<Sender<()>>) -> Result<(), failure::Error> {
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
                        }
                        Err(_) => message::user_error("Something went wrong while watching."),
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
                            if commands::run(command, &command_name).is_ok() {
                                if let Some(tx) = tx.clone() {
                                    let _ = tx.send(());
                                }
                            }
                        }
                        Err(_) => message::user_error("Something went wrong while watching."),
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
