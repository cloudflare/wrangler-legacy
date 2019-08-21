extern crate serde_json;

use cloudflare::endpoints::workerskv::list_namespace_keys::ListNamespaceKeys;
use cloudflare::endpoints::workerskv::list_namespace_keys::ListNamespaceKeysParams;
use cloudflare::endpoints::workerskv::Key;
use cloudflare::framework::apiclient::ApiClient;
use failure::bail;
use serde_json::value::Value as JsonValue;

// Note: this function only prints keys in json form, given that
// the number of entries in each json blob is variable (so csv and tsv
// representation won't make sense)
pub fn list_keys(id: &str, prefix: Option<&str>) -> Result<(), failure::Error> {
    let client = super::api_client()?;
    let account_id = super::account_id()?;

    let params = ListNamespaceKeysParams {
        limit: None, // Defaults to 1000 (the maximum)
        cursor: None,
        prefix: prefix.map(str::to_string),
    };

    let mut request_params = ListNamespaceKeys {
        account_identifier: &account_id,
        namespace_identifier: id,
        params: params,
    };

    let mut response = client.request(&request_params);

    print!("["); // Open json list bracket

    // used to track whether to put a glue "," before printing a json blob.
    let mut page = 1;
    // Iterate over all pages until no pages of keys are left.
    // This is detected when a returned cursor is an empty string.
    // todo(gabbi): the code in this loop is the product of a looooong fight
    // with the borrow checker. Please tell me if there's a neater way to write
    // the logic below!
    loop {
        let (result, cursor) = match response {
            Ok(success) => (
                success.result,
                get_cursor_from_result_info(success.result_info.clone()),
            ),
            Err(e) => bail!(e),
        };

        match cursor {
            None => {
                // Case where we are done iterating through pages (no cursor returned)
                print_page(result, page)?;
                print!("]"); // Close json list bracket
                break;
            }
            Some(_) => {
                // Case where we still have pages to iterate through (a cursor is returned).
                // Update cursor in request_params.params, and make another request to Workers KV API.
                request_params.params.cursor = cursor;

                // todo(gabbi): Right now, I print out the results of every page
                // as wrangler gets them (instead of storing them in memory and
                // outputting them all at once). What do the reviewers think about this?
                // I figured this was the best option because it wouldn't eat memory, but
                // I'm curious what other folks think.
                print_page(result, page)?;
                response = client.request(&request_params);
            }
        }
        page = page + 1;
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

fn print_page(keys: Vec<Key>, page: isize) -> Result<(), failure::Error> {
    // add comma between this set of json blobs and the previous json blob, if
    // previous json blob exists. This "concatenates" them.
    if page > 1 {
        print!(",")
    }
    for key in keys {
        print!("{}", serde_json::to_string(&key)?);
    }
    Ok(())
}
