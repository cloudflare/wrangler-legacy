use serde::Serialize;

#[derive(Serialize, Debug)]
#[serde(tag = "type")]
pub enum Binding {
    #[allow(non_camel_case_types)]
    wasm_module { name: String, part: String },
}

impl Binding {
    pub fn new_wasm_module(name: &str, part: &str) -> Binding {
        Binding::wasm_module {
            name: name.to_string(),
            part: part.to_string(),
        }
    }
}
