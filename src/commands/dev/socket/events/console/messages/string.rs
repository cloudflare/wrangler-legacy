use serde::{Deserialize, Serialize};
use std::fmt;

#[serde_with::skip_serializing_none]
#[derive(Debug, Serialize, Deserialize)]
pub struct StringData {
    #[serde(default)]
    value: String,
    description: Option<String>,
}

impl fmt::Display for StringData {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match &self.description {
            Some(d) => write!(f, "\"{}\"", d),
            None => write!(f, "\"{}\"", &self.value),
        }
    }
}