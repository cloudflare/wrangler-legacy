use super::binding::Binding;
use super::wasm_module::WasmModule;

#[derive(Debug)]
pub struct ProjectAssets {
    pub script_path: String,
    pub wasm_modules: Vec<WasmModule>,
}

impl ProjectAssets {
    pub fn bindings(&self) -> Vec<Binding> {
        let mut bindings = Vec::new();

        for wm in &self.wasm_modules {
            let wasm = wm.binding();
            bindings.push(wasm);
        }

        bindings
    }

    pub fn script_name(&self) -> String {
        "script".to_string() // TODO: derive from script_path
    }

    pub fn script_path(&self) -> String {
        self.script_path.to_string()
    }
}
