use crate::commands;
use crate::terminal::message::{Message, StdOut};
use crate::terminal::{interactive, styles};

use prettytable::{Cell, Row, Table};

static SCOPES_LIST: [&str; 8] = [
    "account:read",
    "user:read",
    "workers:write",
    "workers_kv:write",
    "workers_routes:write",
    "workers_scripts:write",
    "workers_tail:read",
    "zone:read",
];

static DESCRIPTION_LIST: [&str; 8] = [
    "see your account info such as account details, analytics, and memberships.",
    "see your user info such as name, email address, and account memberships.",
    "see and change Cloudflare Workers data such as zones, KV storage, namespaces, scripts, and routes.",
    "see and change Cloudflare Workers KV Storage data such as keys and namespaces.",
    "see and change Cloudflare Workers data such as filters and routes.",
    "see and change Cloudflare Workers scripts, durable objects, subdomains, triggers, and tail data.",
    "see Cloudflare Workers tail and script data.",
    "Grants read level access to account zone."
];

/// Format OAuth scopes into a nice table
fn format_scopes(
    scopes: &[&str],
    descriptions: &[&str] 
) -> Table {
    let mut table = Table::new();
    let table_head = Row::new(vec![Cell::new("Scope"), Cell::new("Description")]);
    table.add_row(table_head);

    for (scope, description) in scopes.iter().zip(descriptions.iter()) {
        let row = Row::new(vec![Cell::new(scope), Cell::new(description)]);
        table.add_row(row);
    }
    table
}

pub fn login(scopes: bool) -> Result<(), anyhow::Error> {
    if scopes {
        StdOut::info(&format!("Available scopes \n\n{}", format_scopes(SCOPES_LIST.as_ref(), DESCRIPTION_LIST.as_ref())));
        let scopes_input: String = interactive::get_user_input("Please enter desired scopes in a whitespace separated list (e.g. \"scope1 scope2 .. scopeN\").");
        let scopes_list: Vec<&str> = scopes_input.split_whitespace().collect();

        if !scopes_list.is_empty() {
            return commands::login::run(Some(scopes_list.as_ref()));
        }

        // User didn't provide any scopes
        StdOut::message(&format!(
            "No scope provided. {} will be configured with the default scopes.",
            styles::highlight("`wrangler login`")
        ));
    }

    // If the user doesn't provide flag or doesn't enter any scopes, fallback to default scopes
    commands::login::run(None)
}
