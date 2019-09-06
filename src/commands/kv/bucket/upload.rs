use std::fs::metadata;
use std::path::Path;

use cloudflare::endpoints::workerskv::write_bulk::KeyValuePair;

use crate::commands::kv::bucket::directory_keys_values;
use crate::commands::kv::bulk::put::put_bulk;
use crate::settings::global_user::GlobalUser;
use crate::settings::project::Project;

pub fn upload(
    project: &Project,
    user: GlobalUser,
    namespace_id: &str,
    filename: &Path,
) -> Result<(), failure::Error> {
    let pairs: Result<Vec<KeyValuePair>, failure::Error> = match &metadata(filename) {
        Ok(file_type) if file_type.is_dir() => directory_keys_values(filename),
        Ok(_file_type) => {
            // any other file types (files, symlinks)
            failure::bail!("wrangler kv:bucket upload takes a directory")
        }
        Err(e) => failure::bail!("{}", e),
    };

    put_bulk(project, user, namespace_id, pairs?)
}
