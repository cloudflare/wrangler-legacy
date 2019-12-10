use serde::{Deserialize, Serialize};
use std::fmt;

use crate::commands::dev::socket::events::console::LogMessage;

#[derive(Debug, Serialize, Deserialize)]
pub struct ArrayData {
    pub preview: ArrayPreview,
    pub description: String,
}

impl fmt::Display for ArrayData {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{} {}", &self.description, &self.preview)
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ArrayPreview {
    pub properties: Vec<LogMessage>,
}

impl fmt::Display for ArrayPreview {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "[")?;
        let last_index = self.properties.len() - 1;
        for (index, property) in &mut self.properties.iter().enumerate() {
            if index == 0 {}
            write!(f, "{}", property)?;
            if index < last_index {
                write!(f, ", ")?;
            }
        }
        write!(f, "]")?;
        Ok(())
    }
}
