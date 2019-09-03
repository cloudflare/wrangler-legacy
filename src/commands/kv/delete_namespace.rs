use cloudflare::framework::apiclient::ApiClient;

use cloudflare::endpoints::workerskv::remove_namespace::RemoveNamespace;

use crate::commands::kv;
use crate::terminal::message;

pub fn delete_namespace(id: &str, force: bool) -> Result<(), failure::Error> {
    let client = kv::api_client()?;
    let account_id = kv::account_id()?;

    if !force {
        match kv::interactive_delete(&format!(
            "Are you sure you want to delete namespace {}?",
            id
        )) {
            Ok(true) => (),
            Ok(false) => {
                message::info(&format!("Not deleting namespace {}", id));
                return Ok(());
            }
            Err(e) => failure::bail!(e),
        }
    }

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
