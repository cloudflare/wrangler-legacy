extern crate serde_json;

use cloudflare::endpoints::workerskv::list_namespace_keys::ListNamespaceKeys;
use cloudflare::endpoints::workerskv::list_namespace_keys::ListNamespaceKeysParams;
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
    let mut subsequent_result = false;
    // Iterate over all pages until no pages of keys are left.
    // This is detected when a returned cursor is an empty string.
    // todo(gabbi): the code in this loop is the product of a looooong fight
    // with the borrow checker. Please tell me if there's a neater way to write
    // the logic below!
    loop {
        let (result, cursor) = match response {
            Ok(ref success) => (
                serde_json::to_string(&success.result.clone())?,
                get_cursor_from_result_info(success.result_info.clone()),
            ),
            Err(e) => bail!(e),
        };

        match cursor {
            None => {
                // Case where we are done iterating through pages (no cursor returned)
                print_page(&result, subsequent_result);
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
                print_page(&result, subsequent_result);
                response = client.request(&request_params);
            }
        }
        subsequent_result = true;
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

fn print_page(json_string: &str, is_subsequent: bool) {
    // add comma between this set of json blobs and the previous json blob, if
    // previous json blob exists. This "concatenates" them.
    if is_subsequent {
        print!(",")
    }
    // don't print out beginning`[` and ending `]` braces; the point is that we want only one
    // json array to get returned (so we print out each page's json blobs only
    // and add the `[]` array notation ourselves).
    print!("{}", &json_string[1..json_string.len() - 1]);
}
