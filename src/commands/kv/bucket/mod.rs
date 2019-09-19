extern crate base64;

mod delete;
mod sync;
mod upload;

pub use delete::delete;
pub use sync::sync;
pub use upload::upload;

use std::ffi::OsString;
use std::path::Path;

use cloudflare::endpoints::workerskv::write_bulk::KeyValuePair;

use walkdir::WalkDir;

use crate::terminal::message;

fn directory_keys_values(
    directory: &Path,
    verbose: bool,
) -> Result<Vec<KeyValuePair>, failure::Error> {
    let mut upload_vec: Vec<KeyValuePair> = Vec::new();
    for entry in WalkDir::new(directory) {
        let entry = entry.unwrap();
        let path = entry.path();
        if path.is_file() {
            let key = generate_key(path, directory)?;

            let value = std::fs::read(path)?;

            // Need to base64 encode value
            let b64_value = base64::encode(&value);
            if verbose {
                message::working(&format!("Parsing {}...", key.clone()));
            }
            upload_vec.push(KeyValuePair {
                key,
                value: b64_value,
                expiration: None,
                expiration_ttl: None,
                base64: Some(true),
            });
        }
    }
    Ok(upload_vec)
}

fn directory_keys_only(directory: &Path) -> Result<Vec<String>, failure::Error> {
    let mut upload_vec: Vec<String> = Vec::new();
    for entry in WalkDir::new(directory) {
        let entry = entry.unwrap();
        let path = entry.path();
        if path.is_file() {
            let key = generate_key(path, directory)?;

            upload_vec.push(key);
        }
    }
    Ok(upload_vec)
}

// Courtesy of Steve Kalabnik's PoC :) Used for bulk operations (write, delete)
fn generate_key(path: &Path, directory: &Path) -> Result<String, failure::Error> {
    let path = path.strip_prefix(directory).unwrap();

    // next, we have to re-build the paths: if we're on Windows, we have paths with
    // `\` as separators. But we want to use `/` as separators. Because that's how URLs
    // work.
    let mut path_with_forward_slash = OsString::new();

    for (i, component) in path.components().enumerate() {
        // we don't want a leading `/`, so skip that
        if i > 0 {
            path_with_forward_slash.push("/");
        }

        path_with_forward_slash.push(component);
    }

    // if we have a non-utf8 path here, it will fail, but that's not realistically going to happen
    let path = path_with_forward_slash
        .to_str()
        .unwrap_or_else(|| panic!("found a non-UTF-8 path, {:?}", path_with_forward_slash));

    Ok(path.to_string())
}
