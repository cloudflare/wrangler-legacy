use serde::Serialize;
use std::path::PathBuf;

#[derive(Serialize, Debug)]
pub struct WasmModule {
    #[serde(skip_serializing)]
    pub path: PathBuf,
    pub name: String,
    pub part: String,
}

#[derive(Serialize, Debug)]
#[serde(tag = "type")]
pub enum Binding {
    #[allow(non_camel_case_types)]
    wasm_module(WasmModule),
}

impl Binding {
    pub fn new_wasm_module(path: PathBuf, name: String, part: String) -> Binding {
        Binding::wasm_module(WasmModule { path, name, part })
    }
}
