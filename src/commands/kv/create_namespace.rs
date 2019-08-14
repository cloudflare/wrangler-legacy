use cloudflare::apiclient::ApiClient;

use cloudflare::workerskv::create_namespace::CreateNamespace;
use cloudflare::workerskv::create_namespace::CreateNamespaceParams;

use crate::terminal::message;

pub fn create_namespace(title: &str) -> Result<(), failure::Error> {
    let client = super::api_client()?;
    let account_id = super::account_id()?;

    let msg = format!("Creating namespace with title \"{}\"", title);
    message::working(&msg);

    let response = client.request(&CreateNamespace {
        account_identifier: &account_id,
        params: CreateNamespaceParams {
            title: title.to_string(),
        },
    });

    match response {
        Ok(success) => message::success(&format!("Success: {:#?}", success.result)),
        Err(e) => super::print_error(e),
    }

    Ok(())
}
