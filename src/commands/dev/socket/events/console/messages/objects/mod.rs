mod object;
mod subtype;

use serde::{Deserialize, Serialize};
use std::fmt;

use object::Object;
use subtype::Subtype;

#[derive(Debug, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ObjectData {
    Subtype(Subtype),
    Object(Object),
}

impl fmt::Display for ObjectData {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match &self {
            ObjectData::Object(object) => {
                let last_index = object.preview.properties.len() - 1;
                for (idx, property) in &mut object.preview.properties.iter().enumerate() {
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
            }
            ObjectData::Subtype(subtype) => {
                write!(f, "{}", subtype)?;
            }
        };
        Ok(())
    }
}
