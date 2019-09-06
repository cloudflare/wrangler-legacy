use cloudflare::endpoints::workerskv::list_namespaces::ListNamespaces;
use cloudflare::endpoints::workerskv::WorkersKvNamespace;
use cloudflare::framework::apiclient::ApiClient;

use prettytable::{Cell, Row, Table};

use crate::commands::kv;
use crate::settings::global_user::GlobalUser;
use crate::settings::project::Project;
use crate::terminal::message;

pub fn list(project: &Project, user: GlobalUser) -> Result<(), failure::Error> {
    let client = kv::api_client(user)?;

    message::working("Fetching namespaces...");

    let response = client.request(&ListNamespaces {
        account_identifier: &project.account_id,
    });

    match response {
        Ok(success) => {
            let table = namespace_table(success.result);
            message::success(&format!("Success: \n{}", table));
        }
        Err(e) => kv::print_error(e),
    }

    Ok(())
}

fn namespace_table(namespaces: Vec<WorkersKvNamespace>) -> Table {
    let mut table = Table::new();
    let table_head = Row::new(vec![Cell::new("TITLE"), Cell::new("ID")]);

    table.add_row(table_head);
    for ns in namespaces {
        let row = Row::new(vec![Cell::new(&ns.title), Cell::new(&ns.id)]);
        table.add_row(row);
    }

    table
}
