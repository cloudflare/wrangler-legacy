use super::binding::{Binding, ToBinding};
use super::file::{File, ToFile};
use super::script::Script;
use super::wasm_module::WasmModule;

#[derive(Debug)]
pub struct ProjectAssets {
    pub script: Script,
    pub wasm_modules: Vec<WasmModule>,
}

impl ProjectAssets {
    pub fn files(&self) -> Vec<File> {
        let mut files = Vec::new();
        let script = self.script.to_file();
        files.push(script);
        for wm in &self.wasm_modules {
            let wasm = wm.to_file();
            files.push(wasm);
        }

        files
    }

    pub fn bindings(&self) -> Vec<Binding> {
        let mut bindings = Vec::new();
        for wm in &self.wasm_modules {
            let wasm = wm.to_binding();
            bindings.push(wasm);
        }

        bindings
    }
}
