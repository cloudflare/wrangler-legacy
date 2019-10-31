use std::path::PathBuf;

use failure::format_err;

use super::binding::Binding;
use super::filename_from_path;

#[derive(Debug)]
pub struct WasmModule {
    path: PathBuf,
    filename: String,
    binding: String,
}

impl WasmModule {
    pub fn new(path: PathBuf, binding: String) -> Result<Self, failure::Error> {
        let filename = filename_from_path(&path)
            .ok_or_else(|| format_err!("filename should not be empty: {:?}", path))?;

        Ok(Self {
            filename,
            path,
            binding,
        })
    }

    // `name` corresponds to the binding used in the worker js
    // `part` corresponds to the name given to the file in the upload form
    pub fn binding(&self) -> Binding {
        let name = &self.binding;
        let part = &self.filename;

        Binding::new_wasm_module(name, part)
    }

    pub fn path(&self) -> PathBuf {
        self.path.clone()
    }

    pub fn filename(&self) -> String {
        self.filename.to_string()
    }
}
