mod messages;

use messages::LogMessage;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct ConsoleEvent {
    #[serde(rename = "type")]
    pub log_type: String,
    #[serde(rename = "args")]
    pub messages: Vec<LogMessage>,
}
