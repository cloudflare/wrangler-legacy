use notify::DebouncedEvent;
use std::collections::HashSet;
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
    match event {
        DebouncedEvent::Error(error, _) => Err(format_err!("{:?}", error)),
        DebouncedEvent::NoticeWrite(path) => Ok(filter_ignored(path, ignore_dirs)),
        DebouncedEvent::Write(path) => Ok(filter_ignored(path, ignore_dirs)),
        DebouncedEvent::NoticeRemove(path) => Ok(filter_ignored(path, ignore_dirs)),
        DebouncedEvent::Remove(path) => Ok(filter_ignored(path, ignore_dirs)),
        DebouncedEvent::Create(path) => Ok(filter_ignored(path, ignore_dirs)),
        _ => Ok(None),
    }
}

// Exclude files within ignored directories from live reload.
fn filter_ignored(path: PathBuf, ignore_dirs_opt: Option<&HashSet<String>>) -> Option<PathBuf> {
    // let path_str = path.to_str();
    // if path_str.is_none() {
    //     return None;
    // }

    // let mut absolute_path: Vec<&str> = path_str.unwrap().split("/./").collect();
    // let relative_path = absolute_path.pop();
    // if relative_path.is_none() {
    //     return None;
    // }

    

    // Check if file in ignored directory.
    if let Some(ignore_dirs) = ignore_dirs_opt {
        for ignore_dir in ignore_dirs {
            if let Some(relative) = relative_path {
                if relative.starts_with(ignore_dir) {
                    return None;
                }
            }
        }
    }
    Some(path)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;
    use std::path::Path;

    #[test]
    fn it_can_filter_ignored_path() {
        let to_ignore = ["pkg", "target", "worker/generated"];
        let ignore_dirs: HashSet<_> = to_ignore.iter().map(|d| d.to_string()).collect();
        let test_path = "home/blah/worker/generated/file.txt";
        assert!(filter_ignored(Path::new(test_path).to_path_buf(), Some(&ignore_dirs)).is_none());
    }

    #[test]
    fn it_can_include_src_file() {
        let to_ignore = ["pkg", "target", "worker/generated"];
        let ignore_dirs: HashSet<_> = to_ignore.iter().map(|d| d.to_string()).collect();
        let test_path = "home/blah/src/lib.rs";
        assert!(filter_ignored(Path::new(test_path).to_path_buf(), Some(&ignore_dirs)).is_some());
    }
}
