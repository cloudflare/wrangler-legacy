use super::binding::Binding;
use super::file::File;
use super::wasm_module::WasmModule;

#[derive(Debug)]
pub struct ProjectAssets {
    pub script_path: String,
    pub wasm_modules: Vec<WasmModule>,
}

impl ProjectAssets {
    pub fn files(&self) -> Vec<File> {
        let mut files = Vec::new();
        let script = File {
            name: "script".to_string(),
            path: self.script_path.clone(),
        };
        files.push(script);

        for wm in &self.wasm_modules {
            let wasm = wm.file();
            files.push(wasm);
        }

        files
    }

    pub fn bindings(&self) -> Vec<Binding> {
        let mut bindings = Vec::new();

        for wm in &self.wasm_modules {
            let wasm = wm.binding();
            bindings.push(wasm);
        }

        bindings
    }
}
