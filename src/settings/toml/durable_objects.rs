use serde::{Deserialize, Serialize};

use crate::settings::binding::Binding;

#[derive(Clone, Debug, Default, Deserialize, PartialEq, Serialize)]
pub struct DurableObjects {
    pub uses: Option<Vec<DurableObjectNamespace>>,
    pub implements: Option<Vec<DurableObjectNamespaceImpl>>,
}

// TODO(now): Are there reasonable defaults we can come up with for namespace_name/binding?
// The bash script uses <script-name>-<class-name> for the namespace name,
// and <class-name> for the binding.

#[derive(Clone, Debug, Default, Deserialize, PartialEq, Serialize)]
pub struct DurableObjectNamespace {
    pub binding: String,
    pub namespace_name: Option<String>,
    pub namespace_id: Option<String>,
}

impl DurableObjectNamespace {
    pub fn binding(&self) -> Result<Binding, failure::Error> {
        match &self.namespace_id {
            Some(namespace_id) => Ok(Binding::new_durable_object_namespace(
                self.binding.clone(),
                namespace_id.clone(),
            )),
            None => Err(failure::err_msg(format!(
                "id not found or provided for durable object namespace bound to {}",
                self.binding
            ))),
        }
    }
}

#[derive(Clone, Debug, Default, Deserialize, PartialEq, Serialize)]
pub struct DurableObjectNamespaceImpl {
    pub namespace_name: String,
    pub class_name: String,
}
