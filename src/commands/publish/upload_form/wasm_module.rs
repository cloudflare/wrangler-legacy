use super::binding::{Binding, ToBinding};
use super::file::{File, ToFile};

#[derive(Debug)]
pub struct WasmModule {
    pub path: String,
    pub filename: String,
    pub binding: String,
}

impl ToBinding for WasmModule {
    fn to_binding(&self) -> Binding {
        let name = self.filename.clone();
        let part = self.binding.clone();

        Binding::new_wasm_module(name, part)
    }
}

impl ToFile for WasmModule {
    fn to_file(&self) -> File {
        File {
            name: self.filename.clone(),
            path: self.path.clone(),
        }
    }
}
