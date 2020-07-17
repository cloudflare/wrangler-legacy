use std::collections::HashSet;
use std::path::Path;

use cloudflare::endpoints::workerskv::write_bulk::KeyValuePair;

use crate::commands::kv;
use crate::http;
use crate::kv::key::KeyList;
use crate::settings::global_user::GlobalUser;
use crate::settings::toml::Target;
use crate::terminal::message;

use super::directory_keys_values;
use super::manifest::AssetManifest;

pub fn sync(
    target: &Target,
    user: &GlobalUser,
    namespace_id: &str,
    path: &Path,
) -> Result<(Vec<KeyValuePair>, Vec<String>, AssetManifest), failure::Error> {
    kv::validate_target(target)?;
    // First, find all changed files in given local directory (aka files that are now stale
    // in Workers KV).

    // Get remote keys, which contain the hash of the file (value) as the suffix.
    // Turn it into a HashSet. This will be used by upload() to figure out which
    // files to exclude from upload (because their current version already exists in
    // the Workers KV remote).
    let client = http::cf_v4_client(&user)?;
    let remote_keys_iter = KeyList::new(target, client, namespace_id, None)?;
    let mut remote_keys: HashSet<String> = HashSet::new();
    for remote_key in remote_keys_iter {
        match remote_key {
            Ok(remote_key) => {
                remote_keys.insert(remote_key.name);
            }
            Err(e) => failure::bail!(kv::format_error(e)),
        }
    }

    let (pairs, asset_manifest): (Vec<KeyValuePair>, AssetManifest) =
        directory_keys_values(target, path)?;

    let to_upload = filter_files(pairs.clone(), &remote_keys);

    // Now delete files from Workers KV that exist in remote but no longer exist locally.
    // Get local keys
    let mut local_keys: HashSet<_> = HashSet::new();
    for pair in pairs.iter() {
        local_keys.insert(pair.key.clone());
    }

    // Find keys that are present in remote but not present in local, and
    // stage them for deletion.
    let to_delete: Vec<_> = remote_keys
        .difference(&local_keys)
        .map(|key| key.to_owned())
        .collect();

    message::success("Success");
    Ok((to_upload, to_delete, asset_manifest))
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
    use crate::sites::generate_path_and_key;
    use std::collections::HashSet;
    use std::path::Path;

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
        assert_eq!(expected.len(), actual.len());
        for (idx, pair) in expected.into_iter().enumerate() {
            // Ensure the expected key and value was returned in the filtered pair list
            // Awkward field-by-field comparison below courtesy of not yet implementing
            // PartialEq for KeyValuePair in cloudflare-rs :)
            // TODO: (gabbi) Implement PartialEq for KeyValuePair in cloudflare-rs.
            assert_eq!(pair.key, actual[idx].key);
            assert_eq!(pair.value, actual[idx].value);
        }
    }
}
