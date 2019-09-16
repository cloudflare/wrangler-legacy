use std::fs::metadata;
use std::path::Path;

use cloudflare::endpoints::workerskv::write_bulk::KeyValuePair;

use crate::commands::kv::bucket::directory_keys_values;
use crate::commands::kv::bulk::put::put_bulk;
use crate::settings::global_user::GlobalUser;
use crate::settings::target::Target;
use crate::terminal::message;

const KEY_MAX_SIZE: usize = 512;
const VALUE_MAX_SIZE: usize = 2 * 1024 * 1024;
const PAIRS_MAX_COUNT: usize = 10000;
const UPLOAD_MAX_SIZE: usize = 100 * 1024 * 1024;

pub fn upload(
    target: &Target,
    user: GlobalUser,
    namespace_id: &str,
    path: &Path,
) -> Result<(), failure::Error> {
    let mut pairs: Vec<KeyValuePair> = match &metadata(path) {
        Ok(file_type) if file_type.is_dir() => directory_keys_values(path),
        Ok(_file_type) => {
            // any other file types (files, symlinks)
            failure::bail!("wrangler kv:bucket upload takes a directory")
        }
        Err(e) => failure::bail!("{}", e),
    }?;

    validate_file_uploads(pairs.clone())?;

    // Create a vector of uploads; that is, a vector of vectors of key-value pairs, each of which are
    // maximum 10K key-value pairs in size OR maximum 100MB in size.
    let mut key_count = 0;
    let mut key_pair_bytes = 0;
    let mut key_value_batch: Vec<KeyValuePair> = Vec::new();

    while !(pairs.is_empty() && key_value_batch.is_empty()) {
        if pairs.is_empty() {
            // Last batch to upload
            call_put_bulk_api(target, user.clone(), namespace_id, &mut key_value_batch)?;
        } else {
            let pair = pairs.pop().unwrap();
            if key_count + pair.key.len() > PAIRS_MAX_COUNT // Max KV pairs for request met
            || key_pair_bytes + pair.key.len() + pair.value.len() > UPLOAD_MAX_SIZE * 80 / 100
            // key+value sums nearly at UPLOAD_MAX_SIZE
            {
                call_put_bulk_api(target, user.clone(), namespace_id, &mut key_value_batch)?;

                // If upload successful, reset counters
                key_count = 0;
                key_pair_bytes = 0;
            }

            // Add the popped key-value pair to the running batch of key-value pair uploads
            key_count = key_count + pair.key.len();
            key_pair_bytes = key_pair_bytes + pair.key.len() + pair.value.len();
            key_value_batch.push(pair);
        }
    }

    message::success("Success");
    Ok(())
}

fn call_put_bulk_api(
    target: &Target,
    user: GlobalUser,
    namespace_id: &str,
    key_value_batch: &mut Vec<KeyValuePair>,
) -> Result<(), failure::Error> {
    message::info("Uploading...");
    // If partial upload fails (e.g. server error), return that error message
    put_bulk(target, user.clone(), namespace_id, key_value_batch.clone())?;

    key_value_batch.clear();
    Ok(())
}

// Ensure that all key-value pairs being uploaded have valid sizes (this ensures that
// no partial uploads happen). I don't like this function because it duplicates the
// size checking the API already does--but doing a preemptive check like this (before
// calling the API) will prevent partial bucket uploads from happening.
pub fn validate_file_uploads(pairs: Vec<KeyValuePair>) -> Result<(), failure::Error> {
    for pair in pairs {
        if pair.key.len() > KEY_MAX_SIZE {
            failure::bail!(
                "Path `{}` exceeds the maximum key size limit of {} bytes",
                pair.key,
                KEY_MAX_SIZE
            );
        }
        if pair.key.len() > KEY_MAX_SIZE {
            failure::bail!(
                "File `{}` exceeds the maximum value size limit of {} bytes",
                pair.key,
                VALUE_MAX_SIZE
            );
        }
    }
    Ok(())
}
