use crate::commands;
use crate::terminal::message::{Message, StdOut};

use prettytable::{Cell, Row, Table};

// List of valid OAuth scopes
pub static SCOPES_LIST: [&str; 8] = [
    "account:read",
    "user:read",
    "workers:write",
    "workers_kv:write",
    "workers_routes:write",
    "workers_scripts:write",
    "workers_tail:read",
    "zone:read",
];

// Description for each scope in SCOPES_LIST
static DESCRIPTION_LIST: [&str; 8] = [
    "See your account info such as account details, analytics, and memberships.",
    "See your user info such as name, email address, and account memberships.",
    "See and change Cloudflare Workers data such as zones, KV storage, namespaces, scripts, and routes.",
    "See and change Cloudflare Workers KV Storage data such as keys and namespaces.",
    "See and change Cloudflare Workers data such as filters and routes.",
    "See and change Cloudflare Workers scripts, durable objects, subdomains, triggers, and tail data.",
    "See Cloudflare Workers tail and script data.",
    "Grants read level access to account zone.",
];

/// Format OAuth scopes into a nice table
fn format_scopes(scopes: &[&str], descriptions: &[&str]) -> Table {
    let mut table = Table::new();
    let table_head = Row::new(vec![Cell::new("Scope"), Cell::new("Description")]);
    table.add_row(table_head);

    for (scope, description) in scopes.iter().zip(descriptions.iter()) {
        let row = Row::new(vec![Cell::new(scope), Cell::new(description)]);
        table.add_row(row);
    }
    table
}

pub fn login(scopes: &[String], scopes_list: bool) -> Result<(), anyhow::Error> {
    if scopes_list {
        StdOut::info(&format!(
            "Available scopes \n\n{}",
            format_scopes(SCOPES_LIST.as_ref(), DESCRIPTION_LIST.as_ref())
        ));
        return Ok(());
    }

    // User provided scopes
    if !scopes.is_empty() {
        return commands::login::run(Some(scopes));
    }

    // No user input, default scopes
    commands::login::run(None)
}
