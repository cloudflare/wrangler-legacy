use cloudflare::endpoints::workerskv::remove_namespace::RemoveNamespace;
use cloudflare::framework::apiclient::ApiClient;
use cloudflare::framework::response::{ApiFailure, ApiSuccess};
use cloudflare::framework::HttpApiClient;

pub fn delete(
    client: HttpApiClient,
    account_id: &str,
    id: &str,
) -> Result<ApiSuccess<()>, ApiFailure> {
    client.request(&RemoveNamespace {
        account_identifier: account_id,
        namespace_identifier: id,
    })
}
