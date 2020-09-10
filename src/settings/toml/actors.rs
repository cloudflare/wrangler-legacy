use crate::settings::binding::Binding;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct ConfigActors {
    pub uses: Option<Vec<ConfigActorNamespaceBinding>>,
    pub implements: Option<Vec<ConfigActorNamespaceImpl>>,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct ConfigActorNamespaceBinding {
    pub binding: String,
    pub name: Option<String>,
    pub preview_name: Option<String>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct ActorNamespaceNoId {
    pub binding: String,
    pub name: String,
}

#[derive(Clone, Debug, PartialEq)]
pub struct ActorNamespace {
    pub binding: String,
    pub id: String,
}

impl ActorNamespace {
    pub fn binding(&self) -> Binding {
        Binding::new_actor_namespace(self.binding.clone(), self.id.clone())
    }
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct ConfigActorNamespaceImpl {
    pub name: String,
}
