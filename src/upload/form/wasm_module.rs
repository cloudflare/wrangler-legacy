use std::path::PathBuf;

use failure::format_err;

use super::binding::Binding;
use super::filestem_from_path;

// Note: This is only used for service-worker scripts.
// modules scripts use the universal Module class instead of this.

#[derive(Debug)]
pub struct WasmModule {
    path: PathBuf,
    filename: String,
    binding: String,
}

impl WasmModule {
    pub fn new(path: PathBuf, binding: String) -> Result<Self, failure::Error> {
        let filename = filestem_from_path(&path)
            .ok_or_else(|| format_err!("filename should not be empty: {}", path.display()))?;

        Ok(Self {
            filename,
            path,
            binding,
        })
    }

    // `name` corresponds to the binding used in the worker js
    // `part` corresponds to the name given to the file in the upload form
    pub fn binding(&self) -> Binding {
        Binding::new_wasm_module(self.binding.clone(), self.filename.clone())
    }

    pub fn path(&self) -> PathBuf {
        self.path.clone()
    }

    pub fn filename(&self) -> String {
        self.filename.to_string()
    }
}
