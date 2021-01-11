extern crate serde_json;

use cloudflare::endpoints::workerskv::list_namespaces::ListNamespaces;
use cloudflare::endpoints::workerskv::list_namespaces::ListNamespacesParams;
use cloudflare::endpoints::workerskv::WorkersKvNamespace;
use cloudflare::framework::apiclient::ApiClient;
use cloudflare::framework::response::ApiSuccess;

use serde::Deserialize;

use crate::commands::kv;
use crate::settings::toml::Target;

const MAX_NAMESPACES_PER_PAGE: u32 = 100;

pub fn list(
    client: &impl ApiClient,
    target: &Target,
) -> Result<Vec<WorkersKvNamespace>, failure::Error> {
    let mut namespaces: Vec<WorkersKvNamespace> = Vec::new();
    let mut all_namespaces_added = false;
    let mut page_number = 1;
    while !all_namespaces_added {
        let params = ListNamespacesParams {
            page: Some(page_number),
            per_page: Some(MAX_NAMESPACES_PER_PAGE),
        };

        match client.request(&ListNamespaces {
            account_identifier: &target.account_id,
            params,
        }) {
            Ok(response) => {
                namespaces.append(&mut response.result.clone());
                page_number += 1;
                all_namespaces_added = namespaces.len() >= get_total(&response)?;
            }
            Err(e) => failure::bail!("{}", kv::format_error(e)),
        }
    }
    Ok(namespaces)
}

fn get_total(list_response: &ApiSuccess<Vec<WorkersKvNamespace>>) -> Result<usize, failure::Error> {
    let result_info: ListResponseResultInfo =
        serde_json::from_value(list_response.result_info.clone().unwrap())?;
    Ok(result_info.total_count)
}

#[derive(Deserialize)]
struct ListResponseResultInfo {
    total_count: usize,
}
