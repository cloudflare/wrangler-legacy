use cloudflare::endpoints::workerskv::list_namespaces::ListNamespaces;
use cloudflare::endpoints::workerskv::list_namespaces::ListNamespacesParams;
use cloudflare::endpoints::workerskv::WorkersKvNamespace;
use cloudflare::framework::apiclient::ApiClient;
use cloudflare::framework::response::ApiSuccess;

use anyhow::Result;
use serde::Deserialize;

use crate::commands::kv;
use crate::settings::toml::Target;

const MAX_NAMESPACES_PER_PAGE: u32 = 1000;

pub fn list(client: &impl ApiClient, target: &Target) -> Result<Vec<WorkersKvNamespace>> {
    let mut namespaces: Vec<WorkersKvNamespace> = Vec::new();
    let mut all_namespaces_added = false;
    let mut page_number = 1;
    while !all_namespaces_added {
        let params = ListNamespacesParams {
            page: Some(page_number),
            per_page: Some(MAX_NAMESPACES_PER_PAGE),
        };

        match client.request(&ListNamespaces {
            account_identifier: target.account_id.load()?,
            params,
        }) {
            Ok(response) => {
                namespaces.append(&mut response.result.clone());
                page_number += 1;
                all_namespaces_added = namespaces.len() >= get_total(&response)?;
            }
            Err(e) => anyhow::bail!("{}", kv::format_error(e)),
        }
    }
    Ok(namespaces)
}

fn get_total(list_response: &ApiSuccess<Vec<WorkersKvNamespace>>) -> Result<usize> {
    match list_response.result_info.clone() {
        Some(r) => {
            let result_info: ListResponseResultInfo = serde_json::from_value(r)?;
            Ok(result_info.total_count)
        }
        None => anyhow::bail!("KV list response lacks result_info field"),
    }
}

#[derive(Deserialize)]
struct ListResponseResultInfo {
    total_count: usize,
}
