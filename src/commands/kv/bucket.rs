extern crate base64;

use crate::commands::kv::delete_bulk::delete_bulk;
use crate::commands::kv::write_bulk::write_bulk;

use cloudflare::endpoints::workerskv::write_bulk::KeyValuePair;

use walkdir::WalkDir;

use std::ffi::OsString;
use std::fs::metadata;
use std::path::Path;

use failure::bail;

use crate::terminal::message;

pub fn upload(namespace_id: &str, filename: &Path) -> Result<(), failure::Error> {
    let pairs: Result<Vec<KeyValuePair>, failure::Error> = match metadata(filename) {
        Ok(ref file_type) if file_type.is_dir() => parse_directory(filename),
        Ok(_file_type) => {
            // any other file types (files, symlinks)
            bail!("wrangler kv:bucket upload takes a directory",)
        }
        Err(e) => bail!(e),
    };

    write_bulk(namespace_id, pairs?)
}

pub fn delete(namespace_id: &str, filename: &Path) -> Result<(), failure::Error> {
    let keys: Result<Vec<String>, failure::Error> = match metadata(filename) {
        Ok(ref file_type) if file_type.is_dir() => {
            let key_value_pairs = parse_directory(filename)?;
            Ok(key_value_pairs
                .iter()
                .map(|kv| kv.key.clone())
                .collect::<Vec<_>>())
        }
        Ok(_) => {
            // any other file types (namely, symlinks)
            bail!(
                "{} should be a file or directory, but is a symlink",
                filename.display()
            )
        }
        Err(e) => bail!(e),
    };

    delete_bulk(namespace_id, keys?)
}

fn parse_directory(directory: &Path) -> Result<Vec<KeyValuePair>, failure::Error> {
    let mut upload_vec: Vec<KeyValuePair> = Vec::new();
    for entry in WalkDir::new(directory) {
        let entry = entry.unwrap();
        let path = entry.path();
        if path.is_file() {
            let key = generate_key(path, directory)?;

            let value = std::fs::read(path)?;

            // Need to base64 encode value
            let b64_value = base64::encode(&value);
            message::working(&format!("Parsing {}...", key.clone()));
            upload_vec.push(KeyValuePair {
                key: key,
                value: b64_value,
                expiration: None,
                expiration_ttl: None,
                base64: Some(true),
            });
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
    let path = path_with_forward_slash.to_str().expect(&format!(
        "found a non-UTF-8 path, {:?}",
        path_with_forward_slash
    ));

    Ok(path.to_string())
}
