use std::collections::HashSet;
use std::path::Path;

use anyhow::Result;
use cloudflare::endpoints::workerskv::write_bulk::KeyValuePair;

use super::directory_keys_values;
use super::manifest::AssetManifest;
use crate::commands::kv;
use crate::http;
use crate::kv::key::KeyList;
use crate::settings::global_user::GlobalUser;
use crate::settings::toml::Target;
use crate::terminal::message::{Message, StdErr};

pub fn sync(
    target: &Target,
    user: &GlobalUser,
    namespace_id: &str,
    path: &Path,
) -> Result<(Vec<KeyValuePair>, Vec<String>, AssetManifest)> {
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
            Err(e) => anyhow::bail!(kv::format_error(e)),
        }
    }

    let (pairs, asset_manifest, _): (Vec<KeyValuePair>, AssetManifest, _) =
        directory_keys_values(target, path, Some(&remote_keys))?;

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

    StdErr::success("Success");
    Ok((pairs, to_delete, asset_manifest))
}
