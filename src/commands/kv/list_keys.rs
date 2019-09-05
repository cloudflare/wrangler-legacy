extern crate serde_json;

use cloudflare::endpoints::workerskv::list_namespace_keys::ListNamespaceKeys;
use cloudflare::endpoints::workerskv::list_namespace_keys::ListNamespaceKeysParams;
use cloudflare::endpoints::workerskv::Key;
use cloudflare::framework::apiclient::ApiClient;
use serde_json::value::Value as JsonValue;

use crate::commands::kv;
use crate::settings::global_user::GlobalUser;
use crate::settings::project::Project;

// Note: this function only prints keys in json form, given that
// the number of entries in each json blob is variable (so csv and tsv
// representation won't make sense)
pub fn list_keys(
    project: &Project,
    user: GlobalUser,
    id: &str,
    prefix: Option<&str>,
) -> Result<(), failure::Error> {
    let client = kv::api_client(user)?;

    let params = ListNamespaceKeysParams {
        limit: None, // Defaults to 1000 (the maximum)
        cursor: None,
        prefix: prefix.map(str::to_string),
    };

    let mut request_params = ListNamespaceKeys {
        account_identifier: &project.account_id,
        namespace_identifier: id,
        params: params,
    };

    let mut response = client.request(&request_params);

    print!("["); // Open json list bracket

    // Iterate over all pages until no pages of keys are left.
    // This is detected when a returned cursor is an empty string.
    loop {
        let (result, cursor) = match response {
            Ok(success) => (
                success.result,
                get_cursor_from_result_info(success.result_info.clone()),
            ),
            Err(e) => failure::bail!(e),
        };

        match cursor {
            None => {
                // Case where we are done iterating through pages (no cursor returned)
                print_page(result, true)?;
                print!("]"); // Close json list bracket
                break;
            }
            Some(_) => {
                // Case where we still have pages to iterate through (a cursor is returned).
                print_page(result, false)?;

                // Update cursor in request_params.params, and make another request to Workers KV API.
                request_params.params.cursor = cursor;
                response = client.request(&request_params);
            }
        }
    }

    Ok(())
}

// Returns Some(cursor) if cursor is non-empty, otherwise returns None.
fn get_cursor_from_result_info(result_info: Option<JsonValue>) -> Option<String> {
    let result_info = result_info.unwrap();
    let returned_cursor_value = &result_info["cursor"];
    let returned_cursor = returned_cursor_value.as_str().unwrap().to_string();
    if returned_cursor.is_empty() {
        None
    } else {
        Some(returned_cursor)
    }
}

fn print_page(keys: Vec<Key>, last_page: bool) -> Result<(), failure::Error> {
    for i in 0..keys.len() {
        print!("{}", serde_json::to_string(&keys[i])?);
        // if last key on last page, don't print final comma.
        if !(last_page && i == keys.len() - 1) {
            print!(",");
        }
    }
    Ok(())
}
