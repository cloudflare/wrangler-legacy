use cloudflare::apiclient::APIClient;
use cloudflare::workerskv::list_namespaces::ListNamespaces;
use cloudflare::workerskv::WorkersKVNamespace;

use prettytable::{Cell, Row, Table};

use crate::terminal::message;

pub fn list_namespaces() -> Result<(), failure::Error> {
    let client = super::api_client()?;
    let account_id = super::account_id()?;

    message::working("Fetching namespaces...");

    let response = client.request(&ListNamespaces {
        account_identifier: &account_id,
    });

    match response {
        Ok(success) => {
            let table = namespace_table(success.result);
            message::success(&format!("Success: \n{}", table));
        }
        Err(e) => super::print_error(e),
    }

    Ok(())
}

fn namespace_table(namespaces: Vec<WorkersKVNamespace>) -> Table {
    let mut table = Table::new();
    let table_head = Row::new(vec![Cell::new("TITLE"), Cell::new("ID")]);

    table.add_row(table_head);
    for ns in namespaces {
        let row = Row::new(vec![Cell::new(&ns.title), Cell::new(&ns.id)]);
        table.add_row(row);
    }

    table
}
