use cloudflare::framework::apiclient::ApiClient;

use cloudflare::endpoints::workerskv::remove_namespace::RemoveNamespace;

use crate::commands::kv;
use crate::terminal::message;

pub fn delete_namespace(id: &str) -> Result<(), failure::Error> {
    let client = kv::api_client()?;
    let account_id = kv::account_id()?;

    let msg = format!("Deleting namespace {}", id);
    message::working(&msg);

    let response = client.request(&RemoveNamespace {
        account_identifier: &account_id,
        namespace_identifier: id,
    });

    match response {
        Ok(_success) => message::success("Success"),
        Err(e) => kv::print_error(e),
    }

    Ok(())
}
