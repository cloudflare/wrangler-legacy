use serde::Serialize;
use ws::{CloseCode, Handler, Handshake, Result as WSResult, Sender};
use crate::terminal::message;

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
    fn on_open(&mut self, handshake: Handshake) -> WSResult<()> {
        #[cfg(not(debug_assertions))]
        const SAFE_ORIGINS: &[&str] = &[
            "https://cloudflareworkers.com/",
        ];

        #[cfg(debug_assertions)]
        const SAFE_ORIGINS: &[&str] = &[
            "https://cloudflareworkers.com/",
            "https://localhost", //trailing slash ommitted to allow for any port
        ];

        let origin = handshake.request.origin()?.unwrap_or("unknown");

        let is_safe = SAFE_ORIGINS.iter().fold(false, |is_safe, safe_origin| {
            is_safe || origin.starts_with(safe_origin)
        });

        if is_safe {
            message::info(&format!("Accepted connection from {}", origin));
        } else {
            message::user_error(&format!(
                "Denied connection from {}. This is not a trusted origin",
                origin
            ));

            let _ = self
                .out
                .close(CloseCode::Policy)
                .expect("failed to close connection to unsafe origin");
        }

        Ok(())
    }
}
