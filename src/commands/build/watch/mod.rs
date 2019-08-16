mod watcher;
pub use watcher::wait_for_changes;

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

pub const COOLDOWN_PERIOD: Duration = Duration::from_millis(2000);

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

                watcher.watch(&entry, RecursiveMode::NonRecursive).unwrap();
                message::info(&format!("watching {:?}", &entry));

                loop {
                    match wait_for_changes(&watcher_rx, COOLDOWN_PERIOD) {
                        Ok(_path) => {
                            if let Some(tx) = tx.clone() {
                                tx.send(()).expect("--watch change message failed to send");
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

            let package = Package::new("./")?;
            let entry = package.main()?;

            thread::spawn(move || {
                let (watcher_tx, watcher_rx) = channel();
                let mut watcher = watcher(watcher_tx, Duration::from_secs(1)).unwrap();

                let mut path = env::current_dir().expect("current dir");
                path.push("src");

                watcher.watch(&path, RecursiveMode::Recursive).unwrap();
                watcher.watch(&entry, RecursiveMode::NonRecursive).unwrap();
                message::info(&format!("watching {:?} and {:?}", &path, &entry));

                loop {
                    match wait_for_changes(&watcher_rx, COOLDOWN_PERIOD) {
                        Ok(_path) => {
                            let command = command(&args, &binary_path);
                            let command_name = format!("{:?}", command);
                            if commands::run(command, &command_name).is_ok() {
                                if let Some(tx) = tx.clone() {
                                    tx.send(()).expect("--watch change message failed to send");
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
