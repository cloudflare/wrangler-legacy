extern crate serde_json;

use crate::commands::kv;
use crate::commands::kv::key::KeyList;
use crate::settings::global_user::GlobalUser;
use crate::settings::project::Project;

// Note: this function only prints keys in json form, given that
// the number of entries in each json blob is variable (so csv and tsv
// representation won't make sense)
pub fn list(
    project: &Project,
    user: GlobalUser,
    namespace_id: &str,
    prefix: Option<&str>,
) -> Result<(), failure::Error> {
    let client = kv::api_client(user)?;

    let key_list = KeyList::fetch(project, client, namespace_id, prefix)?;

    print!("["); // Open json list bracket

    let mut first_page = true;

    for key in key_list {
        if !(first_page) {
            print!(",");
        } else {
            first_page = false;
        }

        print!("{}", serde_json::to_string(&key)?);
    }
    print!("]"); // Close json list bracket

    Ok(())
}
