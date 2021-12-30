use serde::{Deserialize, Serialize};

use crate::settings::binding::Binding;

#[derive(Clone, Debug, Default, Deserialize, PartialEq, Serialize)]
pub struct Service {
    #[serde(alias = "name")]
    pub binding: String,
    pub service: String,
    pub environment: String,
}

impl Service {
    pub fn binding(&self) -> Binding {
        Binding::new_service(
            self.binding.clone(),
            self.service.clone(),
            self.environment.clone(),
        )
    }
}
