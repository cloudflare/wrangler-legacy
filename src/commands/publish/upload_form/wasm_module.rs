use failure::format_err;

use super::binding::Binding;
use super::filename_from_path;

use std::path::{Path, PathBuf};

#[derive(Debug)]
pub struct WasmModule {
    path: PathBuf,
    filename: String,
    binding: String,
}

impl WasmModule {
    pub fn new<P: Into<PathBuf>>(path: P, binding: String) -> Result<Self, failure::Error> {
        let path: PathBuf = path.into();
        let filename = filename_from_path(&path).ok_or_else(|| {
            format_err!("filename should not be empty: {}", path.to_str().unwrap())
        })?;

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

    pub fn path(&self) -> &Path {
        self.path.as_ref()
    }

    pub fn filename(&self) -> String {
        self.filename.to_string()
    }
}
