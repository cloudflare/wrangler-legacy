use super::binding::Binding;
use super::file::File;
use super::kv_namespace::KvNamespace;
use super::wasm_module::WasmModule;

#[derive(Debug)]
pub struct ProjectAssets {
    pub script_path: String,
    pub wasm_modules: Vec<WasmModule>,
    pub kv_namespaces: Vec<KvNamespace>,
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
            let file = wm.file();
            files.push(file);
        }

        files
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
}
