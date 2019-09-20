use std::fs::metadata;
use std::path::Path;

use crate::commands::kv::bucket::directory_keys_only;
use crate::commands::kv::bulk::delete::delete_bulk;
use crate::settings::global_user::GlobalUser;
use crate::settings::target::Target;
use crate::terminal::message;

pub fn delete(
    target: &Target,
    user: GlobalUser,
    namespace_id: &str,
    path: &Path,
) -> Result<(), failure::Error> {
    let keys: Result<Vec<String>, failure::Error> = match &metadata(path) {
        Ok(file_type) if file_type.is_dir() => directory_keys_only(path),
        Ok(_) => failure::bail!("{} should be a directory", path.display()),
        Err(e) => failure::bail!("{}", e),
    };

    match delete_bulk(target, user, namespace_id, keys?) {
        Ok(_) => message::success("Success"),
        Err(e) => print!("{}", e),
    }
    Ok(())
}
