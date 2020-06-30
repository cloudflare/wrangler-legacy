use cloudflare::endpoints::workerskv::create_namespace::CreateNamespace;
use cloudflare::endpoints::workerskv::create_namespace::CreateNamespaceParams;
use cloudflare::endpoints::workerskv::WorkersKvNamespace;
use cloudflare::framework::apiclient::ApiClient;
use cloudflare::framework::response::{ApiFailure, ApiSuccess};

pub fn create(
    client: &impl ApiClient,
    account_id: &str,
    title: &str,
) -> Result<ApiSuccess<WorkersKvNamespace>, ApiFailure> {
    client.request(&CreateNamespace {
        account_identifier: account_id,
        params: CreateNamespaceParams {
            title: title.to_string(),
        },
    })
}
