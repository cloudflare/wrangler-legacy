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

const MAX_PAIRS: usize = 10000;

pub fn write_json(namespace_id: &str, filename: &Path) -> Result<(), failure::Error> {
    let pairs: Result<Vec<KeyValuePair>, failure::Error> = match metadata(filename) {
        Ok(ref file_type) if file_type.is_file() => {
            let data = fs::read_to_string(filename)?;
            Ok(serde_json::from_str(&data)?)
        }
        Ok(_) => bail!("{} should be a JSON file, but is not", filename.display()),
        Err(e) => bail!(e),
    };

    write_bulk(namespace_id, pairs?)
}

pub fn write_bucket(namespace_id: &str, filename: &Path) -> Result<(), failure::Error> {
    let pairs: Result<Vec<KeyValuePair>, failure::Error> = match metadata(filename) {
        Ok(ref file_type) if file_type.is_dir() => parse_directory(filename),
        Ok(_) => bail!("{} should be a directory, but is not", filename.display()),
        Err(e) => bail!(e),
    };

    write_bulk(namespace_id, pairs?)
}

fn write_bulk(namespace_id: &str, pairs: Vec<KeyValuePair>) -> Result<(), failure::Error> {
    let client = super::api_client()?;
    let account_id = super::account_id()?;

    // Validate that bulk upload is within size constraints
    if pairs.len() > MAX_PAIRS {
        bail!(
            "Number of key-value pairs to upload ({}) exceeds max of {}",
            pairs.len(),
            MAX_PAIRS
        );
    }

    message::working("Parsing successful. Uploading all files above");

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
