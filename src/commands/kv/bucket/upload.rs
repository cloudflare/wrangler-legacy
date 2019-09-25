use std::collections::HashSet;
use std::fs::metadata;
use std::path::Path;

use crate::commands::kv::bucket::directory_keys_values;
use crate::commands::kv::bulk::put::put_bulk;
use crate::settings::global_user::GlobalUser;
use crate::settings::target::Target;
use crate::terminal::message;
use cloudflare::endpoints::workerskv::write_bulk::KeyValuePair;
use failure::format_err;

const KEY_MAX_SIZE: usize = 512;
const VALUE_MAX_SIZE: usize = 2 * 1024 * 1024;

// The consts below are halved from the API's true capacity to help avoid
// hammering it with large requests.
const PAIRS_MAX_COUNT: usize = 5000;
const UPLOAD_MAX_SIZE: usize = 50 * 1024 * 1024;

pub fn upload_files(
    target: &Target,
    user: GlobalUser,
    namespace_id: &str,
    path: &Path,
    exclude_keys: Option<&HashSet<String>>,
    verbose: bool,
) -> Result<(), failure::Error> {
    let mut pairs: Vec<KeyValuePair> = match &metadata(path) {
        Ok(file_type) if file_type.is_dir() => {
            let (p, _) = directory_keys_values(path, verbose)?;
            Ok(p)
        }
        Ok(_file_type) => {
            // any other file types (files, symlinks)
            Err(format_err!("wrangler kv:bucket upload takes a directory"))
        }
        Err(e) => Err(format_err!("{}", e)),
    }?;

    let mut ignore = &HashSet::new();
    if let Some(exclude) = exclude_keys {
        ignore = exclude;
    }

    pairs = filter_files(pairs, ignore);

    validate_file_uploads(pairs.clone())?;

    // Iterate over all key-value pairs and create batches of uploads, each of which are
    // maximum 10K key-value pairs in size OR maximum ~50MB in size. Upload each batch
    // as it is created.
    let mut key_count = 0;
    let mut key_pair_bytes = 0;
    let mut key_value_batch: Vec<KeyValuePair> = Vec::new();

    while !(pairs.is_empty() && key_value_batch.is_empty()) {
        if pairs.is_empty() {
            // Last batch to upload
            call_put_bulk_api(target, user.clone(), namespace_id, &mut key_value_batch)?;
        } else {
            let pair = pairs.pop().unwrap();
            if key_count + 1 > PAIRS_MAX_COUNT
            // Keep upload size small to keep KV bulk API happy
            || key_pair_bytes + pair.key.len() + pair.value.len() > UPLOAD_MAX_SIZE
            {
                call_put_bulk_api(target, user.clone(), namespace_id, &mut key_value_batch)?;

                // If upload successful, reset counters
                key_count = 0;
                key_pair_bytes = 0;
            }

            // Add the popped key-value pair to the running batch of key-value pair uploads
            key_count += 1;
            key_pair_bytes = key_pair_bytes + pair.key.len() + pair.value.len();
            key_value_batch.push(pair);
        }
    }

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

    // Can clear batch now that we've uploaded it
    key_value_batch.clear();
    Ok(())
}

fn filter_files(pairs: Vec<KeyValuePair>, already_uploaded: &HashSet<String>) -> Vec<KeyValuePair> {
    let mut filtered_pairs: Vec<KeyValuePair> = Vec::new();
    for pair in pairs {
        if !already_uploaded.contains(&pair.key) {
            filtered_pairs.push(pair);
        }
    }
    filtered_pairs
}

// Ensure that all key-value pairs being uploaded have valid sizes (this ensures that
// no partial uploads happen). I don't like this function because it duplicates the
// size checking the API already does--but doing a preemptive check like this (before
// calling the API) will prevent partial bucket uploads from happening.
pub fn validate_file_uploads(pairs: Vec<KeyValuePair>) -> Result<(), failure::Error> {
    for pair in pairs {
        if pair.key.len() > KEY_MAX_SIZE {
            failure::bail!(
                "Path `{}` of {} bytes exceeds the maximum key size limit of {} bytes",
                pair.key,
                pair.key.len(),
                KEY_MAX_SIZE
            );
        }
        if pair.value.len() > VALUE_MAX_SIZE {
            failure::bail!(
                "File `{}` of {} bytes exceeds the maximum value size limit of {} bytes",
                pair.key,
                pair.value.len(),
                VALUE_MAX_SIZE
            );
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;
    use std::path::Path;

    use cloudflare::endpoints::workerskv::write_bulk::KeyValuePair;

    use crate::commands::kv::bucket::generate_url_safe_key_and_hash;

    #[test]
    fn it_can_filter_preexisting_files() {
        let (_, key_a_old) = generate_url_safe_key_and_hash(
            Path::new("/a"),
            Path::new("/"),
            Some("old".to_string()),
        )
        .unwrap();
        let (_, key_b_old) = generate_url_safe_key_and_hash(
            Path::new("/b"),
            Path::new("/"),
            Some("old".to_string()),
        )
        .unwrap();
        // Generate new key (using hash of new value) for b when to simulate its value being updated.
        let (_, key_b_new) = generate_url_safe_key_and_hash(
            Path::new("/b"),
            Path::new("/"),
            Some("new".to_string()),
        )
        .unwrap();

        // Old values found on remote
        let mut exclude_keys = HashSet::new();
        exclude_keys.insert(key_a_old.clone());
        exclude_keys.insert(key_b_old.clone());

        // local files (with b updated) to upload
        let pairs_to_upload = vec![
            KeyValuePair {
                key: key_a_old,
                value: "old".to_string(), // This value remains unchanged
                expiration_ttl: None,
                expiration: None,
                base64: None,
            },
            KeyValuePair {
                key: key_b_new.clone(),
                value: "new".to_string(), // Note this pair has a new value
                expiration_ttl: None,
                expiration: None,
                base64: None,
            },
        ];

        let expected = vec![KeyValuePair {
            key: key_b_new,
            value: "new".to_string(),
            expiration_ttl: None,
            expiration: None,
            base64: None,
        }];
        let actual = filter_files(pairs_to_upload, &exclude_keys);
        check_kv_pairs_equality(expected, actual);
    }

    fn check_kv_pairs_equality(expected: Vec<KeyValuePair>, actual: Vec<KeyValuePair>) {
        assert!(expected.len() == actual.len());
        let mut idx = 0;
        for pair in expected {
            // Ensure the expected key and value was returned in the filtered pair list
            // Awkward field-by-field comparison below courtesy of not yet implementing
            // PartialEq for KeyValuePair in cloudflare-rs :)
            // todo(gabbi): Implement PartialEq for KeyValuePair in cloudflare-rs.
            assert!(pair.key == actual[idx].key);
            assert!(pair.value == actual[idx].value);
            idx = idx + 1;
        }
    }
}
