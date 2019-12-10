use serde::{Deserialize, Serialize};
use std::fmt;

use crate::commands::dev::socket::events::console::LogMessage;

#[derive(Debug, Serialize, Deserialize)]
pub struct MapData {
    pub preview: MapPreview,
    pub description: String,
}

impl fmt::Display for MapData {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{} {}", &self.description, &self.preview)
        // write!(f, "{}", &self.description)
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MapPreview {
    pub entries: Vec<MapEntry>,
}

impl fmt::Display for MapPreview {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let len = self.entries.len();
        if len > 0 {
            let last_index = len - 1;
            for (index, entry) in &mut self.entries.iter().enumerate() {
                if index == 0 {
                    write!(f, "{{")?;
                }
                write!(f, "{}", entry)?;
                if index < last_index {
                    write!(f, ", ")?;
                } else {
                    write!(f, "}}")?;
                }
            }
        } else {
            write!(f, "{{}}")?;
        }
        Ok(())
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MapEntry {
    pub key: LogMessage,
    pub value: LogMessage,
}

impl fmt::Display for MapEntry {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{} => {}", &self.key, &self.value)
    }
}
