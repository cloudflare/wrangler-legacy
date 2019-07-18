use super::binding::{Binding, ToBinding};
use super::file::{File, ToFile};

#[derive(Debug)]
pub struct WasmModule {
    pub path: String,
    pub filename: String,
    pub binding: String,
}

// `name` corresponds to the binding used in the worker js
// `part` corresponds to the name given to the file in the upload form
impl ToBinding for WasmModule {
    fn to_binding(&self) -> Binding {
        let name = self.binding.clone();
        let part = self.filename.clone();

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
