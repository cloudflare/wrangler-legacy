use std::fs::metadata;
use std::path::Path;

use crate::commands::kv::bucket::directory_keys_only;
use crate::commands::kv::bulk::delete::delete_bulk;
use crate::settings::global_user::GlobalUser;
use crate::settings::project::Project;

pub fn delete(
    project: &Project,
    user: GlobalUser,
    namespace_id: &str,
    filename: &Path,
) -> Result<(), failure::Error> {
    let keys: Result<Vec<String>, failure::Error> = match &metadata(filename) {
        Ok(file_type) if file_type.is_dir() => directory_keys_only(filename),
        Ok(_) => failure::bail!("{} should be a directory", filename.display()),
        Err(e) => failure::bail!("{}", e),
    };

    delete_bulk(project, user, namespace_id, keys?)
}
