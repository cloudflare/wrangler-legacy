use std::fmt;

use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct KVNamespace {
    pub id: String,
    pub binding: String,
}

impl fmt::Display for KVNamespace {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "id: {}, binding: {}", self.id, self.binding)
    }
}

impl std::cmp::PartialEq for KVNamespace {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id && self.binding == other.binding
    }
}
