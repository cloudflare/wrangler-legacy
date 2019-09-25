mod watcher;
pub use watcher::wait_for_changes;

use crate::commands::build::{command, wranglerjs};
use crate::settings::target::{Target, TargetType};
use crate::terminal::message;
use crate::{commands, install};

use notify::{self, RecursiveMode, Watcher};
use std::sync::mpsc;
use std::thread;
use std::time::Duration;

pub const COOLDOWN_PERIOD: Duration = Duration::from_millis(2000);
const JAVASCRIPT_PATH: &str = "./";
const RUST_PATH: &str = "./";

/// watch a project for changes and re-build it when necessary,
/// outputting a build event to tx.
pub fn watch_and_build(
    target: &Target,
    tx: Option<mpsc::Sender<()>>,
) -> Result<(), failure::Error> {
    let target_type = &target.target_type;
    match target_type {
        TargetType::JavaScript => {
            thread::spawn(move || {
                let (watcher_tx, watcher_rx) = mpsc::channel();
                let mut watcher = notify::watcher(watcher_tx, Duration::from_secs(1)).unwrap();

                watcher
                    .watch(JAVASCRIPT_PATH, RecursiveMode::Recursive)
                    .unwrap();
                message::info(&format!("watching {:?}", &JAVASCRIPT_PATH));

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
        TargetType::Rust => {
            let tool_name = "wasm-pack";
            let binary_path = install::install(tool_name, "rustwasm")?.binary(tool_name)?;
            let args = ["build", "--target", "no-modules"];

            thread::spawn(move || {
                let (watcher_tx, watcher_rx) = mpsc::channel();
                let mut watcher = notify::watcher(watcher_tx, Duration::from_secs(1)).unwrap();

                watcher.watch(RUST_PATH, RecursiveMode::Recursive).unwrap();
                message::info(&format!("watching {:?}", &RUST_PATH));

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
        TargetType::Webpack => {
            wranglerjs::run_build_and_watch(&target, tx)?;
        }
    }

    Ok(())
}
