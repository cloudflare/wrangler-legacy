use serde::{Deserialize, Serialize};
use std::fmt::Debug;

#[derive(Debug, Serialize, Deserialize)]
pub struct Message {
    pub method: String,
    pub params: Params,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Params {
    #[serde(rename = "type")]
    pub message_type: String,
    pub args: Vec<LogMessage>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LogMessage {
    #[serde(rename = "type")]
    pub data_type: String,
    pub value: String,
}
