extern crate base64;

use std::fs;
use std::fs::metadata;
use std::path::Path;

use cloudflare::endpoints::workerskv::write_bulk::KeyValuePair;
use cloudflare::endpoints::workerskv::write_bulk::WriteBulk;
use cloudflare::framework::apiclient::ApiClient;
use cloudflare::framework::response::{ApiFailure, ApiSuccess};

use crate::commands::kv;
use crate::commands::kv::bulk::MAX_PAIRS;
use crate::settings::global_user::GlobalUser;
use crate::settings::target::Target;
use crate::terminal::message;

pub fn put(
    target: &Target,
    user: &GlobalUser,
    namespace_id: &str,
    filename: &Path,
) -> Result<(), failure::Error> {
    kv::validate_target(target)?;

    let pairs: Vec<KeyValuePair> = match &metadata(filename) {
        Ok(file_type) if file_type.is_file() => {
            let data = fs::read_to_string(filename)?;
            let data_vec = serde_json::from_str(&data);
            if data_vec.is_err() {
                Err(failure::format_err!("Failed to decode JSON. Please make sure to follow the format, [{{\"key\": \"test_key\", \"value\": \"test_value\"}}, ...]"))
            } else {
                let data_vec: Vec<KeyValuePair> = data_vec.unwrap();
                Ok(data_vec)
            }
        }
        Ok(_) => Err(failure::format_err!(
            "{} should be a JSON file, but is not",
            filename.display()
        )),
        Err(e) => Err(failure::format_err!("{}", e)),
    }?;

    // Validate that bulk upload is within size constraints
    if pairs.len() > MAX_PAIRS {
        failure::bail!(
            "Number of key-value pairs to upload ({}) exceeds max of {}",
            pairs.len(),
            MAX_PAIRS
        );
    }

    let client = kv::api_client(user)?;
    match call_api(&client, target, namespace_id, &pairs) {
        Ok(_) => message::success("Success"),
        Err(e) => failure::bail!("{}", kv::format_error(e)),
    }
    Ok(())
}

pub fn call_api(
    client: &impl ApiClient,
    target: &Target,
    namespace_id: &str,
    pairs: &Vec<KeyValuePair>,
) -> Result<ApiSuccess<()>, ApiFailure> {
    client.request(&WriteBulk {
        account_identifier: &target.account_id,
        namespace_identifier: namespace_id,
        bulk_key_value_pairs: pairs.to_owned(),
    })
}
