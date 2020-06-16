use cloudflare::endpoints::workerskv::create_namespace::CreateNamespace;
use cloudflare::endpoints::workerskv::create_namespace::CreateNamespaceParams;
use cloudflare::endpoints::workerskv::WorkersKvNamespace;
use cloudflare::framework::apiclient::ApiClient;
use cloudflare::framework::response::{ApiFailure, ApiSuccess};

use crate::settings::toml::Target;

pub fn create(
    client: &impl ApiClient,
    target: &Target,
    title: &str,
) -> Result<ApiSuccess<WorkersKvNamespace>, ApiFailure> {
    client.request(&CreateNamespace {
        account_identifier: &target.account_id,
        params: CreateNamespaceParams {
            title: title.to_string(),
        },
    })
}
