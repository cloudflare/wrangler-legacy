use super::binding::Binding;
use serde::{Deserialize, Serialize};

// Note: This is only used for service-worker scripts.
// modules scripts use the universal Module class instead of this.

#[derive(Debug, Deserialize, Serialize)]
pub struct TextBlob {
    pub data: String,
    pub binding: String,
}

impl TextBlob {
    pub fn new(data: String, binding: String) -> Result<Self, failure::Error> {
        Ok(Self { data, binding })
    }

    pub fn binding(&self) -> Binding {
        Binding::new_text_blob(self.binding.clone(), self.binding.clone())
    }
}
