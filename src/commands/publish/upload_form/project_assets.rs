use failure::format_err;

use super::binding::Binding;
use super::filename_from_path;
use super::kv_namespace::KvNamespace;
use super::wasm_module::WasmModule;

use std::path::{Path, PathBuf};

#[derive(Debug)]
pub struct ProjectAssets {
    script_name: String,
    script_path: PathBuf,
    pub wasm_modules: Vec<WasmModule>,
    pub kv_namespaces: Vec<KvNamespace>,
}

impl ProjectAssets {
    pub fn new<P: Into<PathBuf>>(
        script_path: P,
        wasm_modules: Vec<WasmModule>,
        kv_namespaces: Vec<KvNamespace>,
    ) -> Result<Self, failure::Error> {
        let script_path: PathBuf = script_path.into();
        let script_name = filename_from_path(&script_path).ok_or_else(|| {
            format_err!("filename should not be empty: {}", script_path.display())
        })?;

        Ok(Self {
            script_name,
            script_path,
            wasm_modules,
            kv_namespaces,
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

        bindings
    }

    pub fn script_name(&self) -> String {
        self.script_name.to_string()
    }

    pub fn script_path(&self) -> &Path {
        self.script_path.as_ref()
    }
}
