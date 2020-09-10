use crate::http;
use crate::settings::global_user::GlobalUser;
use crate::settings::toml::ActorNamespacesDeployConfig;

use crate::actors::{self, ApiActorNamespace};

pub fn upsert_actor_namespaces(
    user: &GlobalUser,
    script_name: &str,
    actors_config: &ActorNamespacesDeployConfig,
) -> Result<(), failure::Error> {
    let client = http::legacy_auth_client(user);
    log::info!("checking if namespaces are already created");

    let all_namespaces = actors::list_namespaces_by_name_to_id(&client, &actors_config.account_id)?;

    if !all_namespaces.is_empty() {
        let mut new_namespaces = actors_config.actor_namespaces.clone();
        let mut existing_namespaces = Vec::new();

        for i in (0 as usize)..new_namespaces.len() {
            if let Some((name, id)) = all_namespaces.get_key_value(&new_namespaces[i].name) {
                new_namespaces.remove(i);
                existing_namespaces.push(ApiActorNamespace {
                    name: name.clone(),
                    id: id.clone(),
                });
            }
        }

        for namespace in new_namespaces {
            log::info!("creating actor namespace named {}", namespace.name);

            let create_addr = format!(
                "https://api.cloudflare.com/client/v4/accounts/{}/workers/actors/namespaces",
                actors_config.account_id,
            );

            let res = client
                .post(&create_addr)
                .body(
                    serde_json::json!({
                      "name": namespace.name,
                      "script": script_name
                    })
                    .to_string(),
                )
                .header("Content-type", "application/json")
                .send()?;

            if !res.status().is_success() {
                failure::bail!(
                    "Something went wrong! Status: {}, Details {}",
                    res.status(),
                    res.text()?
                )
            }
        }

        for namespace in existing_namespaces {
            log::info!("updating actor namespace named {}", namespace.name);

            let create_addr = format!(
                "https://api.cloudflare.com/client/v4/accounts/{}/workers/actors/namespaces/{}",
                actors_config.account_id, namespace.id
            );

            let res = client
                .put(&create_addr)
                .body(
                    serde_json::json!({
                      "name": namespace.name,
                      "script": script_name
                    })
                    .to_string(),
                )
                .header("Content-type", "application/json")
                .send()?;

            if !res.status().is_success() {
                failure::bail!(
                    "Something went wrong! Status: {}, Details {}",
                    res.status(),
                    res.text()?
                )
            }
        }
    }

    Ok(())
}
