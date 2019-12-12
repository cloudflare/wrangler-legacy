use std::fmt;
use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use crate::settings::binding::Binding;

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct KvNamespace {
    pub id: String,
    pub binding: String,
    pub bucket: Option<PathBuf>,
}

impl fmt::Display for KvNamespace {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "id: {}, binding: {}", self.id, self.binding)
    }
}

impl KvNamespace {
    pub fn binding(&self) -> Binding {
        Binding::new_kv_namespace(self.binding.clone(), self.id.clone())
    }
}
