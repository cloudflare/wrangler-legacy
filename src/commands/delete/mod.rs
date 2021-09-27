pub mod api;

use cloudflare::endpoints::workers::{DeleteDurableObject, DeleteScript};
use cloudflare::framework::apiclient::ApiClient;
use cloudflare::framework::response::ApiFailure;
use cloudflare::framework::HttpApiClient;

use crate::commands::delete::api::fetch_bindings;
use crate::http;
use crate::settings::global_user::GlobalUser;
use crate::terminal::interactive;
use crate::terminal::message::{Message, StdOut};

// Wrapper for `fn script_error`
fn format_error(e: ApiFailure) -> String {
    http::format_error(e, Some(&script_errors))
}

// Provides more detailed explanations of API error codes.
fn script_errors(error_code: u16) -> &'static str {
    match error_code {
        10000 => "Your authentication might be expired or invalid. Please run `wrangler login` or `wrangler config` to authorize Wrangler",
        10007 => "The script could not be found. Please make sure that that the script being deleted exists.",
        10064 => "The Durable Objects namespaces need to be deleted before this script can be deleted.",
        _ => "",
    }
}

// Delete all durable object namespaces used in a script
pub fn delete_durable_objects(
    client: &HttpApiClient,
    account_id: &str,
    script_name: &str,
    force: bool,
) -> Result<(), anyhow::Error> {
    let mut bindings = fetch_bindings(client, account_id, script_name)?;
    bindings.retain(|binding| binding.r#type == "durable_object_namespace");

    if !bindings.is_empty() {
        StdOut::info(&format!(
            "Found {} Durable Object(s) associated with the script {}",
            bindings.len(),
            script_name
        ));

        if !force {
            match interactive::confirm("Are you sure you want to permanently delete the Durable Objects? All the associated data will be permanently LOST.") {
                Ok(true) => (),
                Ok(false) => {
                    return Ok(());
                },
                Err(e) => anyhow::bail!(e),
            }
        }

        // Only delete bound Durable Objects, because they will cause an script_delete API error otherwise
        for binding in bindings {
            if binding.r#type == "durable_object_namespace" {
                let namespace_id = &binding.namespace_id;
                match client.request(&DeleteDurableObject {
                    account_id,
                    namespace_id,
                }) {
                    Ok(_) => StdOut::info(&format!(
                        "Deleted Durable Object - class_name: {}, name: {}",
                        binding.class_name.unwrap(),
                        binding.name
                    )),
                    Err(e) => anyhow::bail!(e),
                }
            }
        }
    }
    Ok(())
}

// Deletes a script_name from an account_id
pub fn delete_script(
    user: &GlobalUser,
    force: bool,
    account_id: &str,
    script_name: &str,
) -> Result<(), anyhow::Error> {
    if !force {
        match interactive::confirm(&format!("Are you sure you want to permanently delete the script name \"{}\" from the account ID {}?", script_name, account_id)) {
            Ok(true) => (),
            Ok(false) => {
                StdOut::info(&format!("Not deleting script \"{}\"", script_name));
                return Ok(());
            },
            Err(e) => anyhow::bail!(e),
        }
    }

    StdOut::working(&format!(
        "Deleting the script \"{}\" on account {}.",
        script_name, account_id
    ));

    let client = http::cf_v4_client(user)?;

    delete_durable_objects(&client, account_id, script_name, force)?;
    match client.request(&DeleteScript {
        account_id,
        script_name,
    }) {
        Ok(_) => {
            StdOut::success(&format!("Success! Deleted script \"{}\".", script_name));
        }
        Err(e) => {
            anyhow::bail!(format_error(e))
        }
    }

    Ok(())
}
