use super::check_file_size;
use std::path::PathBuf;

pub async fn check_wasm(wasm_file: Option<&PathBuf>) -> Result<String, failure::Error> {
    Ok(match wasm_file {
        None => "No WebAssembly detected, skipping check!".to_string(),

        Some(file) => {
            let file_size = check_file_size(file)?;
            format!("WASM size: {}", file_size)
        }
    })
}
