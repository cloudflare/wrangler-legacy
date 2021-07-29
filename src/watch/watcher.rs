use notify::DebouncedEvent;
use std::{
    path::PathBuf,
    sync::mpsc::{Receiver, Sender},
    time::Duration,
};

use anyhow::{anyhow, Result};

use crate::terminal::message::{Message, StdOut};
use log::info;

// Add cooldown for all types of events to watching logic
pub fn wait_for_changes(
    rx: &Receiver<DebouncedEvent>,
    check_channel: Option<Sender<Option<()>>>,
    cooldown: Duration,
) -> Result<PathBuf> {
    loop {
        let event = rx.recv()?;
        // Sending a None to the channel will only succeed if there is a
        // receiver and return from this fn otherwise
        if let Some(check_channel) = &check_channel {
            check_channel.send(None)?;
        }
        match get_changed_path_from_event(event) {
            Ok(Some(path)) => {
                StdOut::working("Detected changes...");
                // wait for cooldown
                while rx.recv_timeout(cooldown).is_ok() {}
                return Ok(path);
            }
            Ok(None) => {
                continue; // was an event type we don't care about, continue
            }
            Err(error) => {
                StdOut::user_error(&format!("WatchError {:?}", error));
                continue;
            }
        };
    }
}

fn get_changed_path_from_event(event: DebouncedEvent) -> Result<Option<PathBuf>> {
    info!("Detected Event {:?}", event);
    match event {
        DebouncedEvent::Error(error, _) => Err(anyhow!(error)),
        DebouncedEvent::NoticeWrite(path) => Ok(Some(path)),
        DebouncedEvent::Write(path) => Ok(Some(path)),
        DebouncedEvent::NoticeRemove(path) => Ok(Some(path)),
        DebouncedEvent::Remove(path) => Ok(Some(path)),
        DebouncedEvent::Create(path) => Ok(Some(path)),
        _ => Ok(None),
    }
}
