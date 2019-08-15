use cloudflare::apiclient::ApiClient;

use cloudflare::workerskv::rename_namespace::RenameNamespace;
use cloudflare::workerskv::rename_namespace::RenameNamespaceParams;

use crate::terminal::message;

pub fn rename_namespace(id: &str, title: &str) -> Result<(), failure::Error> {
    let client = super::api_client()?;
    let account_id = super::account_id()?;

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
        Err(e) => super::print_error(e),
    }

    Ok(())
}
