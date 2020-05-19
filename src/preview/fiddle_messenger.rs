use crate::terminal::message;
use log::info;
use serde::Serialize;
use ws::{CloseCode, Handler, Handshake, Sender};

//for now, this is only used by livereloading.
//in the future we may use this websocket for other things
//so support other message types
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FiddleMessage {
    pub session_id: String,
    #[serde(flatten)]
    pub data: FiddleMessageData,
}

#[derive(Debug, Serialize)]
#[serde(tag = "type")]
pub enum FiddleMessageData {
    #[serde(rename_all = "camelCase")]
    LiveReload { new_id: String },
}

pub struct FiddleMessageServer {
    pub out: Sender,
}

impl Handler for FiddleMessageServer {
    fn on_open(&mut self, handshake: Handshake) -> ws::Result<()> {
        #[cfg(not(debug_assertions))]
        const SAFE_ORIGINS: &[&str] = &["https://cloudflareworkers.com"];

        #[cfg(debug_assertions)]
        const SAFE_ORIGINS: &[&str] = &["https://cloudflareworkers.com", "http://localhost"];

        const SAFE_ADDRS: &[&str] = &["127.0.0.1", "localhost", "::1"];

        // origin() returns Result<Option<&str>>
        let origin = handshake
            .request
            .origin()?
            .unwrap_or("unknown")
            .trim_end_matches(|c: char| c == '/' || c == ':' || c.is_numeric());

        // remote_addr returns Result<Option<String>>
        let incoming_addr = handshake.remote_addr()?;
        let incoming_addr = incoming_addr.as_ref().map_or("unknown", String::as_str);

        // only allow connections from cloudflareworkers.com
        let origin_is_safe = SAFE_ORIGINS
            .iter()
            .any(|safe_origin| &origin == safe_origin);

        // only allow incoming websocket connections from localhost/current machine.
        let addr_is_safe = SAFE_ADDRS
            .iter()
            .any(|safe_addr| &incoming_addr == safe_addr);

        if origin_is_safe && addr_is_safe {
            info!(
                "Accepted connection from site {} incoming from {}",
                origin, incoming_addr
            );
        } else {
            if !origin_is_safe {
                message::user_error(&format!(
                    "Denied connection from site {}. This is not a trusted origin",
                    origin
                ));
            }

            if !addr_is_safe {
                message::user_error(&format!(
                    "Denied connection originating from {} which is outside this machine",
                    incoming_addr
                ));
            }

            self.out
                .close(CloseCode::Policy)
                .expect("failed to close connection to unsafe origin");
        }

        Ok(())
    }
}
