use serde::{Deserialize, Serialize};
use std::fmt;

use crate::commands::dev::socket::events::console::LogMessage;

#[derive(Debug, Serialize, Deserialize)]
pub struct ArrayData {
    pub preview: ArrayPreview,
}

impl fmt::Display for ArrayData {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let last_index = self.preview.properties.len() - 1;
        for (idx, property) in &mut self.preview.properties.iter().enumerate() {
            if idx == 0 {
                write!(f, "[")?;
            }
            write!(f, "{}", property)?;
            if idx < last_index {
                write!(f, ", ")?;
            } else {
                write!(f, "]")?;
            }
        }
        Ok(())
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ArrayPreview {
    properties: Vec<LogMessage>,
}
