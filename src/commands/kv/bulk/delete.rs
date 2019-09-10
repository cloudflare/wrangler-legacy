extern crate base64;

use std::fs;
use std::fs::metadata;
use std::path::Path;

use cloudflare::endpoints::workerskv::delete_bulk::DeleteBulk;
use cloudflare::framework::apiclient::ApiClient;

use crate::commands::kv;
use crate::commands::kv::bulk::MAX_PAIRS;
use crate::settings::global_user::GlobalUser;
use crate::settings::target::Target;
use crate::terminal::message;

pub fn delete(
    target: &Target,
    user: GlobalUser,
    namespace_id: &str,
    filename: &Path,
) -> Result<(), failure::Error> {
    match kv::interactive_delete(&format!(
        "Are you sure you want to delete all keys in {}?",
        filename.display()
    )) {
        Ok(true) => (),
        Ok(false) => {
            message::info(&format!("Not deleting keys in {}", filename.display()));
            return Ok(());
        }
        Err(e) => failure::bail!(e),
    }

    let keys: Result<Vec<String>, failure::Error> = match &metadata(filename) {
        Ok(file_type) if file_type.is_file() => {
            let data = fs::read_to_string(filename)?;
            Ok(serde_json::from_str(&data)?)
        }
        Ok(_) => failure::bail!("{} should be a JSON file, but is not", filename.display()),
        Err(e) => failure::bail!("{}", e),
    };

    delete_bulk(target, user, namespace_id, keys?)
}

fn delete_bulk(
    target: &Target,
    user: GlobalUser,
    namespace_id: &str,
    keys: Vec<String>,
) -> Result<(), failure::Error> {
    let client = kv::api_client(user)?;

    // Check number of pairs is under limit
    if keys.len() > MAX_PAIRS {
        failure::bail!(
            "Number of keys to delete ({}) exceeds max of {}",
            keys.len(),
            MAX_PAIRS
        );
    }

    let response = client.request(&DeleteBulk {
        account_identifier: &target.account_id,
        namespace_identifier: namespace_id,
        bulk_keys: keys,
    });

    match response {
        Ok(_) => message::success("Success"),
        Err(e) => kv::print_error(e),
    }

    Ok(())
}
