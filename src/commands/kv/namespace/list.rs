extern crate serde_json;

use cloudflare::endpoints::workerskv::list_namespaces::ListNamespaces;
use cloudflare::endpoints::workerskv::list_namespaces::ListNamespacesParams;
use cloudflare::endpoints::workerskv::WorkersKvNamespace;
use cloudflare::framework::apiclient::ApiClient;
use cloudflare::framework::response::{ApiFailure, ApiSuccess};

use crate::commands::kv;
use crate::settings::global_user::GlobalUser;
use crate::settings::toml::Target;

const MAX_NAMESPACES_PER_PAGE: u32 = 100;
const PAGE_NUMBER: u32 = 1;

pub fn print_list(target: &Target, user: &GlobalUser) -> Result<(), failure::Error> {
    let namespaces = get_list(target, user)?;
    println!("{}", serde_json::to_string(&namespaces)?);
    Ok(())
}

pub fn get_list(
    target: &Target,
    user: &GlobalUser,
) -> Result<Vec<WorkersKvNamespace>, failure::Error> {
    kv::validate_target(target)?;

    let client = kv::api_client(user)?;
    let result = call_api(&client, target);
    match result {
        Ok(success) => Ok(success.result),
        Err(e) => failure::bail!("{}", kv::format_error(e)),
    }
}

pub fn call_api(
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
