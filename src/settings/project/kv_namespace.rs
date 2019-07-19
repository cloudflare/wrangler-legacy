use std::fmt;

use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct KvNamespace {
    pub id: String,
    pub binding: String,
}

impl fmt::Display for KvNamespace {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "id: {}, binding: {}", self.id, self.binding)
    }
}

impl std::cmp::PartialEq for KvNamespace {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id && self.binding == other.binding
    }
}
