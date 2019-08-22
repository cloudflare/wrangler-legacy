extern crate base64;

use cloudflare::framework::apiclient::ApiClient;
use walkdir::WalkDir;

use std::fs;
use std::fs::metadata;
use std::path::Path;

use cloudflare::endpoints::workerskv::write_bulk::KeyValuePair;
use cloudflare::endpoints::workerskv::write_bulk::WriteBulk;
use failure::bail;

use crate::terminal::message;

const MAX_PAYLOAD_SIZE: usize = 100 * 1024 * 1024; // 100MB
const MAX_KEY_SIZE: usize = 512;
const MAX_VALUE_SIZE: usize = 2 * 1024 * 1024; // 2 MB

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

    // Validate that bulk upload is within size constraints
    let pairs = pairs?;
    // First check number of pairs is under limit
    if pairs.len() > MAX_PAIRS {
        bail!(
            "Number of key-value pairs to upload {} exceeds max of {}",
            pairs.len(),
            MAX_PAIRS
        );
    }
    // Next, iterate over keys and values and make sure each is under limit
    for pair in pairs.clone() {
        if pair.key.len() > MAX_KEY_SIZE {
            bail!(
                "key {} is too large; it is over {} bytes",
                pair.key,
                MAX_KEY_SIZE
            );
        }
        if pair.value.len() > MAX_VALUE_SIZE {
            bail!(
                "value for key {} is too large; it is over {} bytes",
                pair.key,
                MAX_VALUE_SIZE
            );
        }
    }

    let response = client.request(&WriteBulk {
        account_identifier: &account_id,
        namespace_identifier: namespace_id,
        bulk_key_value_pairs: pairs,
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
            let key = super::generate_key(path, directory)?;

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
