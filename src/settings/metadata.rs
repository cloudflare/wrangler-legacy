use serde::Serialize;

use crate::settings::binding::Binding;

#[derive(Serialize, Debug)]
pub struct Metadata {
    pub body_part: String,
    pub bindings: Vec<Binding>,
}
