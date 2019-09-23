use std::collections::HashSet;
use std::fs::metadata;
use std::iter::FromIterator;
use std::path::Path;

use crate::commands::kv;
use crate::commands::kv::bucket::directory_keys_only;
use crate::commands::kv::bucket::upload::upload_files;
use crate::commands::kv::bulk::delete::delete_bulk;
use crate::commands::kv::key::KeyList;
use crate::settings::global_user::GlobalUser;
use crate::settings::target::Target;
use crate::terminal::message;

pub fn sync(
    target: &Target,
    user: GlobalUser,
    namespace_id: &str,
    path: &Path,
    verbose: bool,
) -> Result<(), failure::Error> {
    // First, upload all changed files in given local directory (aka replace files
    // in Workers KV that are now stale).

    // Get remote keys, which contain the hash of the file (value) as the suffix.
    // Turn it into a HashSet. This will be used by upload() to figure out which
    // files to exclude from upload (because their current version already exists in
    // the Workers KV remote).
    let remote_keys_iter = KeyList::new(target, user.clone(), namespace_id, None)?;
    let mut remote_keys: HashSet<String> = HashSet::new();
    for remote_key in remote_keys_iter {
        match remote_key {
            Ok(remote_key) => {
                remote_keys.insert(remote_key.name);
            }
            Err(e) => failure::bail!(kv::format_error(e)),
        }
    }

    if verbose {
        message::info("Preparing to upload updated files...");
    }
    upload_files(
        target,
        user.clone(),
        namespace_id,
        path,
        Some(remote_keys.clone()),
        verbose,
    )?;

    // Now delete files from Workers KV that exist in remote but no longer exist locally.
    // Get local keys
    let local_keys_vec: Vec<String> = match &metadata(path) {
        Ok(file_type) if file_type.is_dir() => directory_keys_only(path),
        Ok(_) => failure::bail!("{} should be a directory", path.display()),
        Err(e) => failure::bail!("{}", e),
    }?;
    let local_keys: HashSet<_> = HashSet::from_iter(local_keys_vec.into_iter());

    // Find keys that are present in remote but not present in local, and
    // stage them for deletion.
    let keys_to_delete: Vec<_> = remote_keys
        .difference(&local_keys)
        .map(|key| key.to_owned())
        .collect();

    if !keys_to_delete.is_empty() {
        if verbose {
            message::info("Deleting stale files...");
        }
        delete_bulk(target, user, namespace_id, keys_to_delete)?;
    }

    message::success("Success");
    Ok(())
}
