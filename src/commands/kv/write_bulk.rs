extern crate base64;

use cloudflare::framework::apiclient::ApiClient;

use std::fs;
use std::fs::metadata;
use std::path::Path;

use cloudflare::endpoints::workerskv::write_bulk::KeyValuePair;
use cloudflare::endpoints::workerskv::write_bulk::WriteBulk;
use failure::bail;

use crate::terminal::message;

const MAX_PAIRS: usize = 10000;

pub fn write_bulk(namespace_id: &str, filename: &Path) -> Result<(), failure::Error> {
    let client = super::api_client()?;
    let account_id = super::account_id()?;

    let pairs: Result<Vec<KeyValuePair>, failure::Error> = match metadata(filename) {
        Ok(ref file_type) if file_type.is_file() => {
            let data = fs::read_to_string(filename)?;
            Ok(serde_json::from_str(&data)?)
        }
        Ok(_file_type) => bail!("wrangler kv write-bulk requires a json file",),
        Err(e) => bail!(e),
    };

    // Validate that bulk upload is within size constraints
    let pairs = pairs?;
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
