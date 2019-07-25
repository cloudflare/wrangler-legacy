use super::binding::Binding;
use super::file::File;

#[derive(Debug)]
pub struct WasmModule {
    pub path: String,
    pub filename: String,
    pub binding: String,
}

// `name` corresponds to the binding used in the worker js
// `part` corresponds to the name given to the file in the upload form
impl WasmModule {
    pub fn binding(&self) -> Binding {
        let name = &self.binding;
        let part = &self.filename;

        Binding::new_wasm_module(name, part)
    }
}

impl WasmModule {
    pub fn file(&self) -> File {
        File {
            name: self.filename.to_string(),
            path: self.path.to_string(),
        }
    }
}
