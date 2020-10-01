use super::check_file_size;
use std::path::PathBuf;

pub fn check_wasm(wasm_file: &PathBuf) -> Result<String, failure::Error> {
    Ok(format!("WASM size: {}", check_file_size(wasm_file)?))
}
