extern crate base64;

use cloudflare::framework::apiclient::ApiClient;
use walkdir::WalkDir;

use std::ffi::OsString;
use std::fs;
use std::fs::metadata;
use std::path::Path;

use cloudflare::endpoints::workerskv::write_bulk::KeyValuePair;
use cloudflare::endpoints::workerskv::write_bulk::WriteBulk;
use failure::bail;

use crate::terminal::message;

pub fn write_bulk(namespace_id: &str, filename: &Path) -> Result<(), failure::Error> {
    let client = super::api_client()?;
    let account_id = super::account_id()?;

    // If the provided argument for write_bulk is a json file, parse it
    // and upload its contents. If the argument is a directory, create key-value
    // pairs where keys are the relative pathnames of files in the directory, and
    // values are the base64-encoded contents of those files.
    let mut data;
    let pairs: Result<Vec<KeyValuePair>, failure::Error> = match metadata(filename) {
        Ok(ref file_type) if file_type.is_file() => {
            data = fs::read_to_string(filename)?;
            Ok(serde_json::from_str(&data)?)
        }
        Ok(ref file_type) if file_type.is_dir() => parse_directory(filename),
        Ok(_file_type) => {
            // any other file types (namely, symlinks)
            bail!(
                "Cannot upload a file that is a symlink: {}",
                filename.display()
            )
        }
        Err(e) => bail!(e),
    };

    let response = client.request(&WriteBulk {
        account_identifier: &account_id,
        namespace_identifier: namespace_id,
        bulk_key_value_pairs: pairs?,
    });

    match response {
        Ok(_success) => message::success("Success"),
        Err(e) => super::print_error(e),
    }

    Ok(())
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
            message::working(&format!("Uploading {}...", key.clone()));
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

// Courtesy of Steve Kalabnik's PoC :)
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
