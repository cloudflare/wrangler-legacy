use std::path::PathBuf;

use failure::format_err;

use super::binding::Binding;
use super::plain_text::PlainText;
use super::text_blob::TextBlob;
use super::wasm_module::WasmModule;
use super::{filename_from_path, filestem_from_path};

use crate::settings::toml::{DurableObjectNamespace, KvNamespace};

#[derive(Debug)]
pub struct ServiceWorkerAssets {
    script_name: String,
    script_path: PathBuf,
    pub wasm_modules: Vec<WasmModule>,
    pub kv_namespaces: Vec<KvNamespace>,
    pub used_durable_object_namespaces: Vec<DurableObjectNamespace>,
    pub text_blobs: Vec<TextBlob>,
    pub plain_texts: Vec<PlainText>,
}

impl ServiceWorkerAssets {
    pub fn new(
        script_path: PathBuf,
        wasm_modules: Vec<WasmModule>,
        kv_namespaces: Vec<KvNamespace>,
        used_durable_object_namespaces: Vec<DurableObjectNamespace>,
        text_blobs: Vec<TextBlob>,
        plain_texts: Vec<PlainText>,
    ) -> Result<Self, failure::Error> {
        let script_name = filestem_from_path(&script_path).ok_or_else(|| {
            format_err!("filename should not be empty: {}", script_path.display())
        })?;

        Ok(Self {
            script_name,
            script_path,
            wasm_modules,
            kv_namespaces,
            used_durable_object_namespaces,
            text_blobs,
            plain_texts,
        })
    }

    pub fn bindings(&self) -> Vec<Binding> {
        let mut bindings = Vec::new();

        for wm in &self.wasm_modules {
            let binding = wm.binding();
            bindings.push(binding);
        }
        for kv in &self.kv_namespaces {
            let binding = kv.binding();
            bindings.push(binding);
        }
        for do_ns in &self.used_durable_object_namespaces {
            let binding = do_ns.binding().expect("namespace id to be provided");
            bindings.push(binding);
        }
        for blob in &self.text_blobs {
            let binding = blob.binding();
            bindings.push(binding);
        }
        for plain_text in &self.plain_texts {
            let binding = plain_text.binding();
            bindings.push(binding);
        }

        bindings
    }

    pub fn script_name(&self) -> String {
        self.script_name.to_string()
    }

    pub fn script_path(&self) -> PathBuf {
        self.script_path.clone()
    }
}

pub struct Module {
    pub path: PathBuf,
    pub module_type: ModuleType,
}

impl Module {
    pub fn new(path: PathBuf) -> Result<Module, failure::Error> {
        let extension = path
            .extension()
            .ok_or_else(|| {
                failure::err_msg(format!(
                    "File {} lacks an extension. An extension is required to determine module type",
                    path.display()
                ))
            })?
            .to_string_lossy();

        let module_type = match extension.as_ref() {
            "mjs" => ModuleType::ES6,
            "js" => ModuleType::CommonJS,
            "wasm" => ModuleType::Wasm,
            "txt" => ModuleType::Text,
            _ => ModuleType::Data,
        };

        Ok(Module { path, module_type })
    }

    pub fn filename(&self) -> Option<String> {
        filename_from_path(&self.path)
    }
}

pub enum ModuleType {
    ES6,
    CommonJS,
    Wasm,
    Text,
    Data,
}

impl ModuleType {
    pub fn content_type(&self) -> &str {
        match &self {
            Self::ES6 => "application/javascript+module",
            Self::CommonJS => "application/javascript",
            Self::Wasm => "application/wasm",
            Self::Text => "text/plain",
            Self::Data => "application/octet-stream",
        }
    }
}

pub struct ModulesAssets {
    pub main_module: String,
    pub modules: Vec<Module>,
    pub kv_namespaces: Vec<KvNamespace>,
    pub used_durable_object_namespaces: Vec<DurableObjectNamespace>,
    pub plain_texts: Vec<PlainText>,
}

impl ModulesAssets {
    pub fn new(
        main_module: String,
        modules: Vec<Module>,
        kv_namespaces: Vec<KvNamespace>,
        used_durable_object_namespaces: Vec<DurableObjectNamespace>,
        plain_texts: Vec<PlainText>,
    ) -> Result<Self, failure::Error> {
        Ok(Self {
            main_module,
            modules,
            kv_namespaces,
            used_durable_object_namespaces,
            plain_texts,
        })
    }

    pub fn bindings(&self) -> Vec<Binding> {
        let mut bindings = Vec::new();

        // Bindings that refer to a `part` of the uploaded files
        // in the service-worker format, are now modules.

        for kv in &self.kv_namespaces {
            let binding = kv.binding();
            bindings.push(binding);
        }
        for do_ns in &self.used_durable_object_namespaces {
            let binding = do_ns.binding().expect("namespace id to be provided");
            bindings.push(binding);
        }
        for plain_text in &self.plain_texts {
            let binding = plain_text.binding();
            bindings.push(binding);
        }

        bindings
    }
}
