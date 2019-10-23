use failure::format_err;

use super::binding::Binding;
use super::filename_from_path;
use super::text_blob::TextBlob;
use super::wasm_module::WasmModule;

use crate::settings::target::KvNamespace;

#[derive(Debug)]
pub struct ProjectAssets {
    script_name: String,
    script_path: String,
    pub wasm_modules: Vec<WasmModule>,
    pub kv_namespaces: Vec<KvNamespace>,
    pub text_blobs: Vec<TextBlob>,
}

impl ProjectAssets {
    pub fn new(
        script_path: String,
        wasm_modules: Vec<WasmModule>,
        kv_namespaces: Vec<KvNamespace>,
        text_blobs: Vec<TextBlob>,
    ) -> Result<Self, failure::Error> {
        let script_name = filename_from_path(&script_path)
            .ok_or_else(|| format_err!("filename should not be empty: {}", script_path))?;

        Ok(Self {
            script_name,
            script_path,
            wasm_modules,
            kv_namespaces,
            text_blobs,
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

        bindings
    }

    pub fn script_name(&self) -> String {
        self.script_name.to_string()
    }

    pub fn script_path(&self) -> String {
        self.script_path.to_string()
    }
}
