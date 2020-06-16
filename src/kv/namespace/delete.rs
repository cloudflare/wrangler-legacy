use cloudflare::endpoints::workerskv::remove_namespace::RemoveNamespace;
use cloudflare::framework::apiclient::ApiClient;
use cloudflare::framework::response::{ApiFailure, ApiSuccess};
use cloudflare::framework::HttpApiClient;

use crate::settings::toml::Target;

pub fn delete(
    client: HttpApiClient,
    target: &Target,
    id: &str,
) -> Result<ApiSuccess<()>, ApiFailure> {
    client.request(&RemoveNamespace {
        account_identifier: &target.account_id,
        namespace_identifier: id,
    })
}
