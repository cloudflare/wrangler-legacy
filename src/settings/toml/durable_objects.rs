use serde::{Deserialize, Serialize};

use crate::settings::binding::Binding;

#[derive(Clone, Debug, Default, Deserialize, PartialEq, Serialize)]
pub struct DurableObjects {
    #[serde(alias = "bindings")]
    pub classes: Option<Vec<DurableObjectsClass>>,
}

#[derive(Clone, Debug, Default, Deserialize, PartialEq, Serialize)]
pub struct DurableObjectsClass {
    #[serde(alias = "name")]
    pub binding: String,
    pub class_name: String,
    pub script_name: Option<String>,
}

impl DurableObjectsClass {
    pub fn binding(&self) -> Binding {
        Binding::new_durable_object_namespace(
            self.binding.clone(),
            self.class_name.clone(),
            self.script_name.clone(),
        )
    }
}
