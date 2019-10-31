use super::manifest::AssetManifest;

use std::collections::HashSet;
use std::fs::metadata;
use std::path::Path;

use crate::commands::kv;
use crate::commands::kv::bucket::directory_keys_values;
use crate::settings::global_user::GlobalUser;
use crate::settings::target::Target;
use crate::terminal::message;
use cloudflare::endpoints::workerskv::write_bulk::KeyValuePair;
use cloudflare::framework::apiclient::ApiClient;
use failure::format_err;

// The consts below are halved from the API's true capacity to help avoid
// hammering it with large requests.
const PAIRS_MAX_COUNT: usize = 5000;
const UPLOAD_MAX_SIZE: usize = 50 * 1024 * 1024;

pub fn upload_files(
    target: &Target,
    user: &GlobalUser,
    namespace_id: &str,
    path: &Path,
    exclude_keys: Option<&HashSet<String>>,
    verbose: bool,
) -> Result<AssetManifest, failure::Error> {
    let (mut pairs, asset_manifest): (Vec<KeyValuePair>, AssetManifest) = match &metadata(path) {
        Ok(file_type) if file_type.is_dir() => {
            let (pairs, asset_manifest) = directory_keys_values(target, path, verbose)?;
            Ok((pairs, asset_manifest))
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

    let client = kv::api_client(user)?;
    // Iterate over all key-value pairs and create batches of uploads, each of which are
    // maximum 10K key-value pairs in size OR maximum ~50MB in size. Upload each batch
    // as it is created.
    let mut key_count = 0;
    let mut key_pair_bytes = 0;
    let mut key_value_batch: Vec<KeyValuePair> = Vec::new();

    while !(pairs.is_empty() && key_value_batch.is_empty()) {
        if pairs.is_empty() {
            // Last batch to upload
            upload_batch(&client, target, namespace_id, &mut key_value_batch)?;
        } else {
            let pair = pairs.pop().unwrap();
            if key_count + 1 > PAIRS_MAX_COUNT
            // Keep upload size small to keep KV bulk API happy
            || key_pair_bytes + pair.key.len() + pair.value.len() > UPLOAD_MAX_SIZE
            {
                upload_batch(&client, target, namespace_id, &mut key_value_batch)?;

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

    Ok(asset_manifest)
}

fn upload_batch(
    client: &impl ApiClient,
    target: &Target,
    namespace_id: &str,
    key_value_batch: &mut Vec<KeyValuePair>,
) -> Result<(), failure::Error> {
    message::info("Uploading...");
    // If partial upload fails (e.g. server error), return that error message
    match kv::bulk::put::call_api(client, target, namespace_id, &key_value_batch) {
        Ok(_) => {
            // Can clear batch now that we've uploaded it
            key_value_batch.clear();
            Ok(())
        }
        Err(e) => failure::bail!("Failed to upload file batch. {}", kv::format_error(e)),
    }
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;
    use std::path::Path;

    use crate::commands::kv::bucket::generate_path_and_key;
    use cloudflare::endpoints::workerskv::write_bulk::KeyValuePair;

    #[test]
    fn it_can_filter_preexisting_files() {
        let (_, key_a_old) =
            generate_path_and_key(Path::new("/a"), Path::new("/"), Some("old".to_string()))
                .unwrap();
        let (_, key_b_old) =
            generate_path_and_key(Path::new("/b"), Path::new("/"), Some("old".to_string()))
                .unwrap();
        // Generate new key (using hash of new value) for b when to simulate its value being updated.
        let (_, key_b_new) =
            generate_path_and_key(Path::new("/b"), Path::new("/"), Some("new".to_string()))
                .unwrap();

        // Old values found on remote
        let mut exclude_keys = HashSet::new();
        exclude_keys.insert(key_a_old.clone());
        exclude_keys.insert(key_b_old);

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
            idx += 1;
        }
    }
}
