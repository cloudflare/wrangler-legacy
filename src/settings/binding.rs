use serde::Serialize;

#[derive(Serialize, Debug)]
#[serde(tag = "type")]
pub enum Binding {
    #[allow(non_camel_case_types)]
    wasm_module { name: String, part: String },
}

impl Binding {
    pub fn new_wasm_module(name: String, part: String) -> Binding {
        Binding::wasm_module { name, part }
    }
}

pub trait ToBinding {
    fn to_binding(&self) -> Binding;
}
