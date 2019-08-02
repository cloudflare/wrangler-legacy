use cloudflare::apiclient::APIClient;

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

    super::print_response(response);

    Ok(())
}
