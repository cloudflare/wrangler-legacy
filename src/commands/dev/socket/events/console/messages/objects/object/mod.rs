mod properties;

use serde::{Deserialize, Serialize};
use std::fmt;

use properties::ObjectProperties;

#[derive(Debug, Serialize, Deserialize)]
pub struct Object {
    pub preview: ObjectPreview,
}

impl fmt::Display for Object {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", &self.preview)
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ObjectPreview {
    #[serde(rename = "type")]
    pub object_type: String,
    pub subtype: Option<String>,
    pub description: String,
    pub overflow: bool,
    pub properties: Vec<ObjectProperties>,
}

impl fmt::Display for ObjectPreview {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if let Some(subtype) = &self.subtype {
            println!("unhandled subtype {}", subtype);
        }
        let len = self.properties.len();
        if len > 0 {
            let last_index = len - 1;
            for (idx, property) in &mut self.properties.iter().enumerate() {
                if idx == 0 {
                    write!(f, "{{")?;
                }
                write!(f, "{}", property)?;
                if idx < last_index {
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
