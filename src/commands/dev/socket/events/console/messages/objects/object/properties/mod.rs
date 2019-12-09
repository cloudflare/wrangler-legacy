use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Debug, Serialize, Deserialize)]
pub struct ObjectProperties {
    pub name: String,
    #[serde(rename = "type")]
    pub property_type: String,
    pub value: String,
}

impl fmt::Display for ObjectProperties {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self.property_type.as_str() {
            "string" => write!(f, "\"{}\": \"{}\"", &self.name, &self.value),
            _ => write!(f, "[{} {}]", &self.property_type, &self.name),
        }
    }
}
