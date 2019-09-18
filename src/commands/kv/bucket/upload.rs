use std::fs::metadata;
use std::path::Path;

use cloudflare::endpoints::workerskv::write_bulk::KeyValuePair;

use crate::commands::kv::bucket::directory_keys_values;
use crate::commands::kv::bulk::put::put_bulk;
use crate::settings::global_user::GlobalUser;
use crate::settings::target::Target;
use crate::terminal::message;

pub fn upload(
    target: &Target,
    user: GlobalUser,
    namespace_id: &str,
    path: &Path,
) -> Result<(), failure::Error> {
    let pairs: Result<Vec<KeyValuePair>, failure::Error> = match &metadata(path) {
        Ok(file_type) if file_type.is_dir() => directory_keys_values(path),
        Ok(_file_type) => {
            // any other file types (files, symlinks)
            failure::bail!("wrangler kv:bucket upload takes a directory")
        }
        Err(e) => failure::bail!("{}", e),
    };

    match put_bulk(target, user, namespace_id, pairs?) {
        Ok(_) => message::success("Success"),
        Err(e) => print!("{}", e),
    }
    Ok(())
}
