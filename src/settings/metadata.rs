use serde::Serialize;

use crate::settings::binding::Binding;

#[derive(Serialize, Debug)]
pub struct Metadata<'a> {
    pub body_part: String,
    pub bindings: &'a Vec<Binding>,
}
