extern crate base64;

use cloudflare::framework::apiclient::ApiClient;

use std::fs;
use std::fs::metadata;
use std::path::Path;

use cloudflare::endpoints::workerskv::delete_bulk::DeleteBulk;
use failure::bail;

use crate::terminal::message;

const MAX_PAIRS: usize = 10000;

pub fn delete_bulk(namespace_id: &str, filename: &Path) -> Result<(), failure::Error> {
    let client = super::api_client()?;
    let account_id = super::account_id()?;

    let keys: Result<Vec<String>, failure::Error> = match metadata(filename) {
        Ok(ref file_type) if file_type.is_file() => {
            let data = fs::read_to_string(filename)?;
            Ok(serde_json::from_str(&data)?)
        }
        Ok(_file_type) => {
            // any other file types (namely, symlinks)
            bail!(
                "{} should be a file or directory, but is a symlink",
                filename.display()
            )
        }
        Err(e) => bail!(e),
    };

    // Validate that bulk delete is within API constraints
    let keys = keys?;
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
