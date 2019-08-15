use cloudflare::apiclient::ApiClient;

use cloudflare::workerskv::remove_namespace::RemoveNamespace;

use crate::terminal::message;

pub fn delete_namespace(id: &str) -> Result<(), failure::Error> {
    let client = super::api_client()?;
    let account_id = super::account_id()?;

    let msg = format!("Deleting namespace {}", id);
    message::working(&msg);

    let response = client.request(&RemoveNamespace {
        account_identifier: &account_id,
        namespace_identifier: id,
    });

    match response {
        Ok(_success) => message::success("Success"),
        Err(e) => super::print_error(e),
    }

    Ok(())
}
