use failure::format_err;

use super::binding::Binding;
use super::filename_from_path;
use super::wasm_module::WasmModule;

#[derive(Debug)]
pub struct ProjectAssets {
    script_name: String,
    script_path: String,
    pub wasm_modules: Vec<WasmModule>,
}

impl ProjectAssets {
    pub fn new(script_path: String, wasm_modules: Vec<WasmModule>) -> Result<Self, failure::Error> {
        let script_name = filename_from_path(&script_path)
            .ok_or(format_err!("filename should not be empty: {}", script_path))?;

        Ok(Self {
            script_name,
            script_path,
            wasm_modules,
        })
    }

    pub fn bindings(&self) -> Vec<Binding> {
        let mut bindings = Vec::new();

        for wm in &self.wasm_modules {
            let wasm = wm.binding();
            bindings.push(wasm);
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
