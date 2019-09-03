use cloudflare::framework::apiclient::ApiClient;

use cloudflare::endpoints::workerskv::rename_namespace::RenameNamespace;
use cloudflare::endpoints::workerskv::rename_namespace::RenameNamespaceParams;

use crate::commands::kv;
use crate::terminal::message;

pub fn rename_namespace(id: &str, title: &str) -> Result<(), failure::Error> {
    let client = kv::api_client()?;
    let account_id = kv::account_id()?;

    let msg = format!("Renaming namespace {} to have title \"{}\"", id, title);
    message::working(&msg);

    let response = client.request(&RenameNamespace {
        account_identifier: &account_id,
        namespace_identifier: &id,
        params: RenameNamespaceParams {
            title: title.to_string(),
        },
    });

    match response {
        Ok(_success) => message::success("Success"),
        Err(e) => kv::print_error(e),
    }

    Ok(())
}
