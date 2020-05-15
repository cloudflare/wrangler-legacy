extern crate base64;

use cloudflare::endpoints::workerskv::delete_bulk::DeleteBulk;
use cloudflare::endpoints::workerskv::write_bulk::KeyValuePair;
use cloudflare::endpoints::workerskv::write_bulk::WriteBulk;
use cloudflare::framework::apiclient::ApiClient;

use crate::commands::kv;
use crate::http;
use crate::settings::global_user::GlobalUser;
use crate::settings::toml::Target;

const MAX_PAIRS: usize = 10000;

pub fn put(
    client: &impl ApiClient,
    target: &Target,
    namespace_id: &str,
    pairs: &[KeyValuePair],
) -> Result<(), failure::Error> {
    // Validate that bulk upload is within size constraints
    if pairs.len() > MAX_PAIRS {
        failure::bail!(
            "Number of key-value pairs to upload ({}) exceeds max of {}",
            pairs.len(),
            MAX_PAIRS
        );
    }

    match client.request(&WriteBulk {
        account_identifier: &target.account_id,
        namespace_identifier: namespace_id,
        bulk_key_value_pairs: pairs.to_owned(),
    }) {
        Ok(_) => Ok(()),
        Err(e) => failure::bail!("{}", kv::format_error(e)),
    }
}

pub fn delete(
    target: &Target,
    user: &GlobalUser,
    namespace_id: &str,
    keys: Vec<String>,
) -> Result<(), failure::Error> {
    let client = http::cf_v4_client(user)?;

    // Check number of pairs is under limit
    if keys.len() > MAX_PAIRS {
        failure::bail!(
            "Number of keys to delete ({}) exceeds max of {}",
            keys.len(),
            MAX_PAIRS
        );
    }

    let response = client.request(&DeleteBulk {
        account_identifier: &target.account_id,
        namespace_identifier: namespace_id,
        bulk_keys: keys,
    });

    match response {
        Ok(_) => Ok(()),
        Err(e) => failure::bail!("{}", kv::format_error(e)),
    }
}
