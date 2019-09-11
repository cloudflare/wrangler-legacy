extern crate base64;

use std::fs;
use std::fs::metadata;
use std::path::Path;

use cloudflare::endpoints::workerskv::write_bulk::KeyValuePair;
use cloudflare::endpoints::workerskv::write_bulk::WriteBulk;
use cloudflare::framework::apiclient::ApiClient;

use crate::commands::kv;
use crate::commands::kv::bulk::MAX_PAIRS;
use crate::settings::global_user::GlobalUser;
use crate::settings::target::Target;
use crate::terminal::message;

pub fn put(
    target: &Target,
    user: GlobalUser,
    namespace_id: &str,
    filename: &Path,
) -> Result<(), failure::Error> {
    let pairs: Result<Vec<KeyValuePair>, failure::Error> = match &metadata(filename) {
        Ok(file_type) if file_type.is_file() => {
            let data = fs::read_to_string(filename)?;
            let data_vec = serde_json::from_str(&data);
            if data_vec.is_err() {
                failure::bail!("Failed to decode JSON. Please make sure to follow the format, [{\"key\": \"test_key\", \"value\": \"test_value\"}, ...]")
            } else {
                Ok(data_vec.unwrap())
            }
        }
        Ok(_) => failure::bail!("{} should be a JSON file, but is not", filename.display()),
        Err(e) => failure::bail!("{}", e),
    };

    put_bulk(target, user, namespace_id, pairs?)
}

fn put_bulk(
    target: &Target,
    user: GlobalUser,
    namespace_id: &str,
    pairs: Vec<KeyValuePair>,
) -> Result<(), failure::Error> {
    let client = kv::api_client(user)?;

    // Validate that bulk upload is within size constraints
    if pairs.len() > MAX_PAIRS {
        failure::bail!(
            "Number of key-value pairs to upload ({}) exceeds max of {}",
            pairs.len(),
            MAX_PAIRS
        );
    }

    let response = client.request(&WriteBulk {
        account_identifier: &target.account_id,
        namespace_identifier: namespace_id,
        bulk_key_value_pairs: pairs,
    });

    match response {
        Ok(_) => message::success("Success"),
        Err(e) => kv::print_error(e),
    }

    Ok(())
}
