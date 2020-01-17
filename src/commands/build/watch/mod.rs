mod watcher;
use ignore::overrides::OverrideBuilder;
use ignore::WalkBuilder;
pub use watcher::wait_for_changes;

use crate::commands::build::{command, wranglerjs};
use crate::settings::toml::{Target, TargetType};
use crate::terminal::message;
use crate::{commands, install};

use notify::{self, RecursiveMode, Watcher};
use std::sync::mpsc;
use std::thread;
use std::time::Duration;

pub const COOLDOWN_PERIOD: Duration = Duration::from_millis(2000);
const JAVASCRIPT_PATH: &str = "./";
const RUST_PATH: &str = "./";

// Paths to ignore live watching in Rust Workers
const RUST_IGNORE: &[&str] = &["pkg", "target", "worker/generated"];

// watch a project for changes and re-build it when necessary,
// outputting a build event to tx.
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
            let tool_author = "rustwasm";
            let is_binary = true;
            let version = install::get_latest_version(tool_name)?;
            let binary_path =
                install::install(tool_name, tool_author, is_binary, version)?.binary(tool_name)?;
            let args = ["build", "--target", "no-modules"];

            thread::spawn(move || {
                let (watcher_tx, watcher_rx) = mpsc::channel();
                let mut watcher = notify::watcher(watcher_tx, Duration::from_secs(1)).unwrap();

                // Populate walker with ignored files so we ensure that the watcher does not watch
                // ignored directories
                let mut ignored_files = OverrideBuilder::new("./");
                for ignore in RUST_IGNORE {
                    ignored_files.add(&format!("!{}", ignore)).unwrap();
                }
                let ignored_file_override = ignored_files.build().unwrap();

                let walker = WalkBuilder::new("./")
                    .overrides(ignored_file_override)
                    .build();

                for entry in walker {
                    let entry = entry.unwrap();
                    if entry.path().is_dir() {
                        continue;
                    }
                    watcher
                        .watch(entry.path(), RecursiveMode::Recursive)
                        .unwrap();
                }
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
            wranglerjs::run_build_and_watch(target, tx)?;
        }
    }

    Ok(())
}
