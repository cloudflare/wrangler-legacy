use notify::DebouncedEvent;
use std::path::PathBuf;
use std::sync::mpsc::Receiver;
use std::time::Duration;

use failure::{format_err, Error};

use crate::terminal::message;
use log::info;

// Add cooldown for all types of events to watching logic
pub fn wait_for_changes(
    rx: &Receiver<DebouncedEvent>,
    cooldown: Duration,
) -> Result<PathBuf, Error> {
    loop {
        let event = rx.recv()?;
        match get_changed_path_from_event(event) {
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

fn get_changed_path_from_event(event: DebouncedEvent) -> Result<Option<PathBuf>, Error> {
    info!("Detected Event {:?}", event);
    match event {
        DebouncedEvent::Error(error, _) => Err(format_err!("{:?}", error)),
        DebouncedEvent::NoticeWrite(path) => Ok(Some(path)),
        DebouncedEvent::Write(path) => Ok(Some(path)),
        DebouncedEvent::NoticeRemove(path) => Ok(Some(path)),
        DebouncedEvent::Remove(path) => Ok(Some(path)),
        DebouncedEvent::Create(path) => Ok(Some(path)),
        _ => Ok(None),
    }
}
