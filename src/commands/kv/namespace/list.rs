extern crate serde_json;

use cloudflare::endpoints::workerskv::list_namespaces::ListNamespaces;
use cloudflare::endpoints::workerskv::list_namespaces::ListNamespacesParams;
use cloudflare::endpoints::workerskv::WorkersKvNamespace;
use cloudflare::framework::apiclient::ApiClient;
use cloudflare::framework::response::{ApiFailure, ApiSuccess};

use crate::commands::kv;
use crate::settings::global_user::GlobalUser;
use crate::settings::target::Target;

const MAX_NAMESPACES_PER_PAGE: u32 = 100;
const PAGE_NUMBER: u32 = 1;

pub fn list(target: &Target, user: &GlobalUser) -> Result<(), failure::Error> {
    kv::validate_target(target)?;
    let result = call_api(target, user);
    match result {
        Ok(success) => {
            let namespaces = success.result;
            println!("{}", serde_json::to_string(&namespaces)?);
        }
        Err(e) => failure::bail!("{}", kv::format_error(e)),
    }
    Ok(())
}

pub fn call_api(
    target: &Target,
    user: &GlobalUser,
) -> Result<ApiSuccess<Vec<WorkersKvNamespace>>, ApiFailure> {
    let client = kv::api_client(user);
    
    let params = ListNamespacesParams {
        page: Some(PAGE_NUMBER),
        per_page: Some(MAX_NAMESPACES_PER_PAGE),
    };

    client.request(&ListNamespaces {
        account_identifier: &target.account_id,
        params,
    })
}
