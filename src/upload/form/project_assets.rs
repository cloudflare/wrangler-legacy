use std::path::PathBuf;

use failure::format_err;

use super::binding::Binding;
use super::filename_from_path;
use super::plain_text::PlainText;
use super::text_blob::TextBlob;
use super::wasm_module::WasmModule;

use crate::settings::toml::KvNamespace;

#[derive(Debug)]
pub struct ServiceWorkerAssets {
    script_name: String,
    script_path: PathBuf,
    pub wasm_modules: Vec<WasmModule>,
    pub kv_namespaces: Vec<KvNamespace>,
    pub text_blobs: Vec<TextBlob>,
    pub plain_texts: Vec<PlainText>,
}

impl ServiceWorkerAssets {
    pub fn new(
        script_path: PathBuf,
        wasm_modules: Vec<WasmModule>,
        kv_namespaces: Vec<KvNamespace>,
        text_blobs: Vec<TextBlob>,
        plain_texts: Vec<PlainText>,
    ) -> Result<Self, failure::Error> {
        let script_name = filename_from_path(&script_path).ok_or_else(|| {
            format_err!("filename should not be empty: {}", script_path.display())
        })?;

        Ok(Self {
            script_name,
            script_path,
            wasm_modules,
            kv_namespaces,
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
