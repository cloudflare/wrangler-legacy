use serde::Serialize;

#[derive(Serialize, Debug)]
#[serde(tag = "type")]
pub enum Binding {
    #[allow(non_camel_case_types)]
    wasm_module { name: String, part: String },
    #[allow(non_camel_case_types)]
    kv_namespace { name: String, namespace_id: String },
}

impl Binding {
    pub fn new_wasm_module(name: String, part: String) -> Binding {
        Binding::wasm_module { name, part }
    }

    pub fn new_kv_namespace(name: String, namespace_id: String) -> Binding {
        Binding::kv_namespace { name, namespace_id }
    }
}
