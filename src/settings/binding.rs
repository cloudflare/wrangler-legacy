use serde::Serialize;

#[derive(Serialize, Debug)]
#[serde(tag = "type")]
#[serde(rename_all = "snake_case")]
pub enum Binding {
    WasmModule {
        name: String,
        part: String,
    },
    KvNamespace {
        name: String,
        namespace_id: String,
    },
    R2Bucket {
        name: String,
        bucket_name: String,
    },
    #[serde(rename = "durable_object_namespace")]
    DurableObjectsClass {
        name: String,
        class_name: String,
        script_name: Option<String>,
    },
    TextBlob {
        name: String,
        part: String,
    },
    PlainText {
        name: String,
        text: String,
    },
}

impl Binding {
    pub fn new_wasm_module(name: String, part: String) -> Binding {
        Binding::WasmModule { name, part }
    }

    pub fn new_kv_namespace(name: String, namespace_id: String) -> Binding {
        Binding::KvNamespace { name, namespace_id }
    }

    pub fn new_r2_bucket(name: String, bucket_name: String) -> Binding {
        Binding::R2Bucket { name, bucket_name }
    }

    pub fn new_durable_object_namespace(
        name: String,
        class_name: String,
        script_name: Option<String>,
    ) -> Binding {
        Binding::DurableObjectsClass {
            name,
            class_name,
            script_name,
        }
    }

    pub fn new_text_blob(name: String, part: String) -> Binding {
        Binding::TextBlob { name, part }
    }

    pub fn new_plain_text(name: String, text: String) -> Binding {
        Binding::PlainText { name, text }
    }
}
