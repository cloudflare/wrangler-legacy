extern crate base64;

use cloudflare::framework::apiclient::ApiClient;
use walkdir::WalkDir;

use std::fs;
use std::fs::metadata;
use std::path::Path;

use cloudflare::endpoints::workerskv::delete_bulk::DeleteBulk;
use failure::bail;

use crate::terminal::message;

const MAX_PAIRS: usize = 10000;

pub fn delete_json(namespace_id: &str, filename: &Path) -> Result<(), failure::Error> {
    let keys: Result<Vec<String>, failure::Error> = match metadata(filename) {
        Ok(ref file_type) if file_type.is_file() => {
            let data = fs::read_to_string(filename)?;
            Ok(serde_json::from_str(&data)?)
        }
        Ok(_) => bail!("{} should be a JSON file, but is not", filename.display()),
        Err(e) => bail!(e),
    };

    delete_bulk(namespace_id, keys?)
}

pub fn delete_directory(namespace_id: &str, filename: &Path) -> Result<(), failure::Error> {
    let keys: Result<Vec<String>, failure::Error> = match metadata(filename) {
        Ok(ref file_type) if file_type.is_dir() => parse_directory(filename),
        Ok(_) => bail!("{} should be a directory, but is not", filename.display()),
        Err(e) => bail!(e),
    };

    delete_bulk(namespace_id, keys?)
}

fn delete_bulk(namespace_id: &str, keys: Vec<String>) -> Result<(), failure::Error> {
    let client = super::api_client()?;
    let account_id = super::account_id()?;

    // Check number of pairs is under limit
    if keys.len() > MAX_PAIRS {
        bail!(
            "Number of keys to delete ({}) exceeds max of {}",
            keys.len(),
            MAX_PAIRS
        );
    }

    let response = client.request(&DeleteBulk {
        account_identifier: &account_id,
        namespace_identifier: namespace_id,
        bulk_keys: keys,
    });

    match response {
        Ok(_success) => message::success("Success"),
        Err(e) => super::print_error(e),
    }

    Ok(())
}

fn parse_directory(directory: &Path) -> Result<Vec<String>, failure::Error> {
    let mut delete_vec: Vec<String> = Vec::new();
    for entry in WalkDir::new(directory) {
        let entry = entry.unwrap();
        let path = entry.path();
        if path.is_file() {
            let key = super::generate_key(path, directory)?;

            message::working(&format!("Going to delete {}...", key.clone()));
            delete_vec.push(key);
        }
    }
    Ok(delete_vec)
}
