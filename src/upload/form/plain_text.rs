use super::binding::Binding;
use anyhow::Result;
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub struct PlainText {
    pub name: String,
    pub value: String,
}

impl PlainText {
    pub fn new(name: String, value: String) -> Result<Self> {
        Ok(Self { name, value })
    }

    pub fn binding(&self) -> Binding {
        Binding::new_plain_text(self.name.clone(), self.value.clone())
    }
}
