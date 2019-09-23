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
    kv::validate_target(target)?;

    // First, upload all existing files in given directory
    if verbose {
        message::info("Preparing to upload updated files...");
    }
    upload_files(target, user.clone(), namespace_id, path, verbose)?;

    // Now delete files from Workers KV that exist in remote but no longer exist locally.
    // Get local keys
    let local_keys_vec: Vec<String> = match &metadata(path) {
        Ok(file_type) if file_type.is_dir() => directory_keys_only(path),
        Ok(_) => failure::bail!("{} should be a directory", path.display()),
        Err(e) => failure::bail!("{}", e),
    }?;
    let local_keys: HashSet<_> = HashSet::from_iter(local_keys_vec.iter());

    // Then get remote keys
    let remote_keys = KeyList::new(target, user.clone(), namespace_id, None)?;

    // Find keys that are present in remote but not present in local, and
    // stage them for deletion. This is done by iterating over the remote_keys and checking for
    // remote_keys that do not exist in local_keys. This logic is similar to that of the
    // difference() method in https://doc.rust-lang.org/src/std/collections/hash/set.rs.html,
    // but saves us the trouble and overhead memory of converting remote_keys into a HashSet.
    let mut keys_to_delete: Vec<String> = Vec::new();
    for remote_key in remote_keys {
        match remote_key {
            Ok(remote_key) => {
                let name = remote_key.name;
                if !local_keys.contains(&name) {
                    keys_to_delete.push(name);
                }
            }
            Err(e) => print!("{}", kv::format_error(e)),
        }
    }

    if !keys_to_delete.is_empty() {
        if verbose {
            message::info("Deleting stale files...");
        }
        delete_bulk(target, user, namespace_id, keys_to_delete)?;
    }

    message::success("Success");
    Ok(())
}
