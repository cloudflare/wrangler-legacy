pub mod api;

use cloudflare::endpoints::account::Account;
use cloudflare::endpoints::workers::{DeleteDurableObject, DeleteScript, WorkersScript};
use cloudflare::framework::apiclient::ApiClient;
use cloudflare::framework::response::ApiFailure;
use cloudflare::framework::HttpApiClient;
use prettytable::{Cell, Row, Table};

use crate::commands::delete::api::{fetch_bindings, fetch_scripts};
use crate::commands::whoami::fetch_accounts;
use crate::http;
use crate::settings::global_user::GlobalUser;
use crate::terminal::interactive;
use crate::terminal::message::{Message, StdOut};

use std::collections::{HashMap, HashSet};

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

// Interactive mode
pub fn run(user: &GlobalUser) -> Result<(), anyhow::Error> {
    // Fetch and display user accounts
    let accounts = fetch_accounts(user)?;
    let (valid_accounts, table) = format_accounts(accounts)?;
    println!("{}", &table);
    let account_name = interactive::get_user_input(
        "Please enter the name of the account you wish to delete a Workers script from",
    );
    if !valid_accounts.contains_key(&account_name) {
        anyhow::bail!("Account name doesn't match.")
    }

    //  Fetch and display scripts related to the account
    let account_id = valid_accounts.get(&account_name).unwrap();
    let scripts = fetch_scripts(user, account_id)?;
    let (valid_scripts, scripts_table) = format_scripts(scripts)?;

    if valid_scripts.is_empty() {
        StdOut::info("There are no scripts associated with the account.");
        return Ok(());
    }
    println!("{}", &scripts_table);
    let script_id =
        interactive::get_user_input("Please enter the name of the Workers script to be deleted.");
    if !valid_scripts.contains(&script_id) {
        anyhow::bail!("Script name doesn't match.")
    }

    // Delete the script
    delete_script(user, false, account_id, &script_id)
}

// Formats the accounts in a table and returns an associated hashtable
fn format_accounts(
    accounts: Vec<Account>,
) -> Result<(HashMap<String, String>, Table), anyhow::Error> {
    let mut valid_accounts = HashMap::with_capacity(accounts.len());
    let mut table = Table::new();
    let table_head = Row::new(vec![Cell::new("Account Name"), Cell::new("Account ID")]);
    table.add_row(table_head);

    for account in accounts {
        let account_lowercase = match account
            .name
            .to_lowercase()
            .split('\'')
            .take(1)
            .next() {
                Some(name) => name.to_string(),
                None => anyhow::bail!("Error while parsing accounts. Please run `wrangler login` and `wrangler delete` again.")
        };

        let row = Row::new(vec![Cell::new(&account_lowercase), Cell::new(&account.id)]);
        table.add_row(row);
        valid_accounts.insert(account_lowercase, account.id);
    }

    if valid_accounts.is_empty() {
        anyhow::bail!("No accounts have been found. You might be missing an \"Account Settings : Read\" permission.")
    }
    Ok((valid_accounts, table))
}

// Formats the scripts in a table and returns an associated hashset
fn format_scripts(scripts: Vec<WorkersScript>) -> Result<(HashSet<String>, Table), anyhow::Error> {
    let mut valid_scripts = HashSet::with_capacity(scripts.len());
    let mut table = Table::new();
    let table_head = Row::new(vec![Cell::new("Script Name")]);
    table.add_row(table_head);

    for script in scripts {
        let row = Row::new(vec![Cell::new(&script.id)]);
        table.add_row(row);
        valid_scripts.insert(script.id);
    }

    Ok((valid_scripts, table))
}

// Delete all durable object namespaces used in a script
pub fn delete_durable_objects(
    client: &HttpApiClient,
    account_id: &str,
    script_id: &str,
    force: bool,
) -> Result<(), anyhow::Error> {
    let mut bindings = fetch_bindings(client, account_id, script_id)?;
    bindings.retain(|binding| binding.r#type == "durable_object_namespace");

    if !bindings.is_empty() {
        StdOut::info(&format!(
            "Found {} Durable Object(s) associated with the script {}",
            bindings.len(),
            script_id
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

// Deletes a script_id from an account_id
pub fn delete_script(
    user: &GlobalUser,
    force: bool,
    account_id: &str,
    script_id: &str,
) -> Result<(), anyhow::Error> {
    if !force {
        match interactive::confirm(&format!("Are you sure you want to permanently delete the script name \"{}\" from the account ID {}?", script_id, account_id)) {
            Ok(true) => (),
            Ok(false) => {
                StdOut::info(&format!("Not deleting script \"{}\"", script_id));
                return Ok(());
            },
            Err(e) => anyhow::bail!(e),
        }
    }

    StdOut::working(&format!(
        "Deleting the script \"{}\" on account {}.",
        script_id, account_id
    ));

    let client = http::cf_v4_client(user)?;

    delete_durable_objects(&client, account_id, script_id, force)?;
    match client.request(&DeleteScript {
        account_id,
        script_id,
    }) {
        Ok(_) => {
            StdOut::success(&format!("Success! Deleted script \"{}\".", script_id));
        }
        Err(e) => {
            anyhow::bail!(format_error(e))
        }
    }

    Ok(())
}
