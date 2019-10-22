use notify::DebouncedEvent;
use std::collections::HashSet;
use std::env;
use std::path::PathBuf;
use std::sync::mpsc::Receiver;
use std::time::Duration;

use failure::{format_err, Error};

use crate::terminal::message;
use log::info;

/// Add cooldown for all types of events to watching logic
pub fn wait_for_changes(
    rx: &Receiver<DebouncedEvent>,
    cooldown: Duration,
    ignore_dirs: Option<&HashSet<String>>,
) -> Result<PathBuf, Error> {
    loop {
        let event = rx.recv()?;
        match get_changed_path_from_event(event, ignore_dirs) {
            Ok(Some(path)) => {
                message::working("Detected changes...");
                // wait for cooldown
                while let Ok(_) = rx.recv_timeout(cooldown) {}
                return Ok(path);
            }
            Ok(None) => {
                continue; // was an event type we don't care about, continue
            }
            Err(error) => {
                message::user_error(&format!("WatchError {:?}", error));
                continue;
            }
        };
    }
}

fn get_changed_path_from_event(
    event: DebouncedEvent,
    ignore_dirs: Option<&HashSet<String>>,
) -> Result<Option<PathBuf>, Error> {
    info!("debounced event is {:?}", event);
    let pwd = env::current_dir()?;
    match event {
        DebouncedEvent::Error(error, _) => Err(format_err!("{:?}", error)),
        DebouncedEvent::NoticeWrite(path) => Ok(filter_ignored(pwd, path, ignore_dirs)?),
        DebouncedEvent::Write(path) => Ok(filter_ignored(pwd, path, ignore_dirs)?),
        DebouncedEvent::NoticeRemove(path) => Ok(filter_ignored(pwd, path, ignore_dirs)?),
        DebouncedEvent::Remove(path) => Ok(filter_ignored(pwd, path, ignore_dirs)?),
        DebouncedEvent::Create(path) => Ok(filter_ignored(pwd, path, ignore_dirs)?),
        _ => Ok(None),
    }
}

// Exclude files within ignored directories from live reload.
fn filter_ignored(
    pwd: PathBuf,
    path: PathBuf,
    ignore_dirs_opt: Option<&HashSet<String>>,
) -> Result<Option<PathBuf>, failure::Error> {
    let path = path.canonicalize()?;
    let relative_path = path.strip_prefix(pwd)?;

    // Check if file in ignored directory.
    if let Some(ignore_dirs) = ignore_dirs_opt {
        for ignore_dir in ignore_dirs {
            if relative_path.starts_with(ignore_dir) {
                return Ok(None);
            }
        }
    }
    Ok(Some(path))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;
    use std::env;
    use std::fs;

    #[test]
    fn it_can_filter_ignored_path() {
        let to_ignore = ["pkg", "target", "worker/generated"];
        let ignore_dirs: HashSet<_> = to_ignore.iter().map(|d| d.to_string()).collect();
        let pwd = env::current_dir().unwrap();
        let test_dir = pwd.join("my-test-worker1");
        let test_path = test_dir.join("worker/generated/file.txt");
        fs::create_dir_all(test_path.clone()).unwrap();

        let outcome = filter_ignored(test_dir.to_path_buf(), test_path, Some(&ignore_dirs));
        fs::remove_dir_all(test_dir).unwrap();
        assert!(outcome.is_ok());
        assert!(outcome.unwrap().is_none());
    }

    #[test]
    fn it_can_include_src_file() {
        let to_ignore = ["pkg", "target", "worker/generated"];
        let ignore_dirs: HashSet<_> = to_ignore.iter().map(|d| d.to_string()).collect();
        let pwd = env::current_dir().unwrap();
        let test_dir = pwd.join("my-test-worker2");
        let test_path = test_dir.join("src/lib.rs");
        fs::create_dir_all(test_path.clone()).unwrap();

        let outcome = filter_ignored(test_dir.to_path_buf(), test_path, Some(&ignore_dirs));
        fs::remove_dir_all(test_dir).unwrap();
        assert!(outcome.is_ok());
        assert!(outcome.unwrap().is_some());
    }
}
