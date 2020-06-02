extern crate serde_json;

use cloudflare::endpoints::workerskv::list_namespaces::ListNamespaces;
use cloudflare::endpoints::workerskv::list_namespaces::ListNamespacesParams;
use cloudflare::endpoints::workerskv::WorkersKvNamespace;
use cloudflare::framework::apiclient::ApiClient;
use cloudflare::framework::response::{ApiFailure, ApiSuccess};

use crate::settings::toml::Target;

const MAX_NAMESPACES_PER_PAGE: u32 = 100;
const PAGE_NUMBER: u32 = 1;

pub fn list(
    client: &impl ApiClient,
    target: &Target,
) -> Result<ApiSuccess<Vec<WorkersKvNamespace>>, ApiFailure> {
    let params = ListNamespacesParams {
        page: Some(PAGE_NUMBER),
        per_page: Some(MAX_NAMESPACES_PER_PAGE),
    };

    client.request(&ListNamespaces {
        account_identifier: &target.account_id,
        params,
    })
}
