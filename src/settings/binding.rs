use serde::Serialize;

#[derive(Serialize, Debug)]
#[serde(tag = "type")]
#[serde(rename_all = "snake_case")]
pub enum Binding {
    WasmModule { name: String, part: String },
    KvNamespace { name: String, namespace_id: String },
    DurableObjectNamespace { name: String, namespace_id: String },
    TextBlob { name: String, part: String },
    PlainText { name: String, text: String },
}

impl Binding {
    pub fn new_wasm_module(name: String, part: String) -> Binding {
        Binding::WasmModule { name, part }
    }

    pub fn new_kv_namespace(name: String, namespace_id: String) -> Binding {
        Binding::KvNamespace { name, namespace_id }
    }

    pub fn new_durable_object_namespace(name: String, namespace_id: String) -> Binding {
        Binding::DurableObjectNamespace { name, namespace_id }
    }

    pub fn new_text_blob(name: String, part: String) -> Binding {
        Binding::TextBlob { name, part }
    }

    pub fn new_plain_text(name: String, text: String) -> Binding {
        Binding::PlainText { name, text }
    }
}
