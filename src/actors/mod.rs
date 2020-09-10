use crate::settings::toml::{ActorNamespace, ActorNamespaceNoId};
use reqwest::blocking::Client;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Serialize, Deserialize, Debug)]
pub struct ApiActorNamespace {
    pub name: String,
    pub id: String,
}

#[derive(Deserialize)]
pub struct ListActorNamespaceResponse {
    pub result: Option<Vec<ApiActorNamespace>>,
}

pub fn list_namespaces_by_name_to_id(
    client: &Client,
    account_id: &str,
) -> Result<HashMap<String, String>, failure::Error> {
    let mut map = HashMap::new();
    let namespaces = list_namespaces(client, account_id)?;
    for namespace in namespaces {
        map.insert(namespace.name, namespace.id);
    }

    Ok(map)
}

pub fn convert_namespace_names_to_ids(
    client: &Client,
    account_id: &str,
    namespaces_noid: &[ActorNamespaceNoId],
) -> Result<Vec<ActorNamespace>, failure::Error> {
    let namespaces = list_namespaces_by_name_to_id(client, account_id)?;
    namespaces_noid.iter().map(|ns| {
        if let Some(id) = namespaces.get(&ns.name) {
            Ok(ActorNamespace {
                binding: ns.binding.clone(),
                id: id.clone()
            })
        } else {
            failure::bail!("actor namespace binding {} is configured with {}, a namespace that does not exist", ns.binding, ns.name);
        }
    }).collect()
}

fn list_namespaces(
    client: &Client,
    account_id: &str,
) -> Result<Vec<ApiActorNamespace>, failure::Error> {
    let list_addr = format!(
        "https://api.cloudflare.com/client/v4/accounts/{}/workers/actors/namespaces",
        account_id,
    );

    let res = client
        .get(&list_addr)
        .header("Content-type", "application/json")
        .send()?;

    if !res.status().is_success() {
        failure::bail!(
            "Something went wrong! Status: {}, Details {}",
            res.status(),
            res.text()?
        )
    }

    Ok(res
        .json::<ListActorNamespaceResponse>()?
        .result
        .unwrap_or_default())
}
