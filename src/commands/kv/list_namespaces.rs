use cloudflare::apiclient::APIClient;

use cloudflare::workerskv::list_namespaces::ListNamespaces;

use crate::terminal::message;

pub fn list_namespaces() -> Result<(), failure::Error> {
    let client = super::api_client()?;
    let account_id = super::account_id()?;

    message::working("Listing namespaces");

    let response = client.request(&ListNamespaces {
        account_identifier: &account_id,
    });

    match response {
        Ok(success) => message::success(&format!("Success: {:#?}", success.result)),
        Err(e) => super::print_error(e),
    }

    Ok(())
}
