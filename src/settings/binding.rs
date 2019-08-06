use serde::Serialize;

#[derive(Serialize, Debug)]
#[serde(tag = "type")]
pub enum Binding {
    #[serde(rename = "wasm_module")]
    WasmModule { name: String, part: String },
    #[serde(rename = "kv_namespace")]
    KvNamespace { name: String, namespace_id: String },
}

impl Binding {
    pub fn new_wasm_module(name: &str, part: &str) -> Binding {
        Binding::WasmModule {
            name: name.to_string(),
            part: part.to_string(),
        }
    }

    pub fn new_kv_namespace(name: String, namespace_id: String) -> Binding {
        Binding::KvNamespace { name, namespace_id }
    }
}
