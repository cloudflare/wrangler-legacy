use std::{cell::RefCell, collections::HashMap};

use crate::settings::toml::DurableObjects;
use crate::settings::{global_user::GlobalUser, toml::ScriptFormat};
use crate::{http, settings::toml::Target};

use reqwest::blocking::Client;
use serde::{Deserialize, Serialize};

/*
Durable Objects have a fun chicken and egg problem that makes them rather tricky to deploy,
in the case of scripts that both implement and bind to a namespace... you must have a namespace
id in order to create the binding to upload the script, but you need the script to create the
namespace. What we did to get around this, was allow creating a namespace without a script,
that just returns an error when you try to access it.

When you use/implement a DO in the same script, we'll first initialize the namespace (if it does
not exist already) with no script/class so we have an ID to use for the binding. After the script is
uploaded, the namespace is finalized with the script and class filled in.

There's only a single DurableObjectsTarget even if there are multiple used durable object
namespaces, or multiple implemented durable object namespaces. This is because we must handle the
(quite common) special case where a single script both implements and uses a durable object
namespace, and it's much easier to handle the initialization/finalization dance if all of the data
necessary to do that is available in one struct.
*/

#[derive(Clone, Debug, PartialEq)]
pub struct DurableObjectsTarget {
    pub account_id: String,
    pub script_name: String,
    pub durable_objects: DurableObjects,
    existing_namespaces: RefCell<HashMap<String, ApiDurableObjectNSResponse>>,
}

#[derive(Serialize, Debug)]
struct DurableObjectCreateNSRequest {
    pub name: String,
    #[serde(flatten)]
    pub implementation: Option<DurableObjectNSImpl>,
}

#[derive(Serialize, Debug)]
struct DurableObjectNSImpl {
    pub script: String,
    pub class: String,
}

impl DurableObjectsTarget {
    pub fn new(account_id: String, script_name: String, durable_objects: DurableObjects) -> Self {
        Self {
            account_id,
            script_name,
            durable_objects,
            existing_namespaces: RefCell::new(HashMap::new()),
        }
    }

    pub fn pre_upload(&self, user: &GlobalUser, target: &mut Target) -> Result<(), failure::Error> {
        if self
            .durable_objects
            .implements
            .as_ref()
            .map_or(false, |i| !i.is_empty())
        {
            match &target.build {
                Some(build) if build.upload_format != ScriptFormat::Modules => {
                    failure::bail!("Implementing a Durable Object namespace requires that the upload_format in your [build] section is set to \"modules\". Please update your wrangler.toml")
                },
                None => failure::bail!("Implementing a Durable Object namespace requires a [build] section to be present in your wrangler.toml with the upload_format set to \"modules\". Please update your wrangler.toml"),
                _ => {}
            }
        }

        self.get_existing_namespaces(user)?;
        self.init_self_referential_namespaces(user)?;
        self.hydrate_target_with_ns_ids(target)?;
        Ok(())
    }

    pub fn only_hydrate(
        &self,
        user: &GlobalUser,
        target: &mut Target,
    ) -> Result<(), failure::Error> {
        self.get_existing_namespaces(user)?;
        self.hydrate_target_with_ns_ids(target)?;
        Ok(())
    }

    pub fn deploy(&self, user: &GlobalUser) -> Result<Vec<String>, failure::Error> {
        let existing_namespaces = self.existing_namespaces.borrow();
        let new_namespaces = self
            .durable_objects
            .implements
            .iter()
            .flatten()
            .filter(|ns| !existing_namespaces.contains_key(&ns.namespace_name));

        let update_namespaces = self
            .durable_objects
            .implements
            .iter()
            .flatten()
            .filter_map(|ns| {
                if let Some(current) = existing_namespaces.get(&ns.namespace_name) {
                    if current.script.as_ref() != Some(&self.script_name)
                        || current.class.as_ref() != Some(&ns.class_name)
                    {
                        Some((current.id.clone(), ns))
                    } else {
                        None
                    }
                } else {
                    None
                }
            });

        let mut updated_namespaces = vec![];

        for ns in new_namespaces {
            updated_namespaces.push(ns.namespace_name.clone());
            create_ns(
                &DurableObjectCreateNSRequest {
                    name: ns.namespace_name.clone(),
                    implementation: Some(DurableObjectNSImpl {
                        script: self.script_name.clone(),
                        class: ns.class_name.clone(),
                    }),
                },
                &self.account_id,
                user,
            )?;
        }

        for (id, ns) in update_namespaces {
            updated_namespaces.push(ns.namespace_name.clone());
            update_ns(
                &id,
                &DurableObjectNSImpl {
                    script: self.script_name.clone(),
                    class: ns.class_name.clone(),
                },
                &self.account_id,
                user,
            )?;
        }

        Ok(updated_namespaces)
    }

    fn get_existing_namespaces(&self, user: &GlobalUser) -> Result<(), failure::Error> {
        log::info!("getting existing Durable Objects");
        let client = http::legacy_auth_client(user);
        self.existing_namespaces
            .replace(list_namespaces_by_name_to_id(&client, &self.account_id)?);
        Ok(())
    }

    fn init_self_referential_namespaces(&self, user: &GlobalUser) -> Result<(), failure::Error> {
        log::info!("initializing self-referential namespaces");
        let implemented_namespace_names = self
            .durable_objects
            .uses
            .iter()
            .flatten()
            .filter_map(|ns| ns.namespace_name.as_ref())
            .collect::<Vec<_>>();

        let existing_namespace_names = {
            let existing_namespaces = self.existing_namespaces.borrow();
            existing_namespaces
                .keys()
                .map(|s| s.to_owned())
                .collect::<Vec<_>>()
        };

        let new_self_referential_namespace_names =
            self.durable_objects.uses.iter().flatten().filter_map(|ns| {
                ns.namespace_name.as_ref().and_then(|name| {
                    if implemented_namespace_names.contains(&name)
                        && !existing_namespace_names.contains(&name)
                    {
                        Some(name)
                    } else {
                        None
                    }
                })
            });

        for name in new_self_referential_namespace_names {
            log::info!("creating error namespace {}", name);
            let new_ns = create_ns(
                &DurableObjectCreateNSRequest {
                    name: name.clone(),
                    implementation: None,
                },
                &self.account_id,
                user,
            )?;
            self.existing_namespaces
                .borrow_mut()
                .insert(new_ns.name.clone(), new_ns);
        }

        Ok(())
    }

    fn hydrate_target_with_ns_ids(&self, target: &mut Target) -> Result<(), failure::Error> {
        for ns in target.used_durable_object_namespaces.iter_mut() {
            if let Some(name) = &ns.namespace_name {
                if ns.namespace_id.is_none() {
                    ns.namespace_id = self
                        .existing_namespaces
                        .borrow()
                        .get(name)
                        .cloned()
                        .map(|ns| ns.id);
                    if ns.namespace_id.is_none() {
                        failure::bail!(format!(
                            "Durable Object namespace with name {} was not found in your account, please check that the name is correct and that you have created the namespace.",
                            name,
                        ))
                    }
                }
            }
        }
        Ok(())
    }
}

fn create_ns(
    body: &DurableObjectCreateNSRequest,
    account_id: &str,
    user: &GlobalUser,
) -> Result<ApiDurableObjectNSResponse, failure::Error> {
    let durable_object_namespace_addr = format!(
        "https://api.cloudflare.com/client/v4/accounts/{}/workers/durable_objects/namespaces",
        account_id
    );

    let client = http::legacy_auth_client(user);
    let res = client
        .post(&durable_object_namespace_addr)
        .header("Content-Type", "application/json")
        .body(serde_json::to_string(body)?)
        .send()?;

    if !res.status().is_success() {
        failure::bail!(
            "Failed to create a Durable Object namespace! Status: {}, Details {}, Body: {:?}",
            res.status(),
            res.text()?,
            body
        )
    }

    match res.json::<ApiSingleDurableObjectNSResponse>()?.result {
        Some(result) => Ok(result),
        None => Err(failure::err_msg(
            "Durable Object not returned from create call despite success status",
        )),
    }
}

fn update_ns(
    namespace_id: &str,
    body: &DurableObjectNSImpl,
    account_id: &str,
    user: &GlobalUser,
) -> Result<(), failure::Error> {
    let durable_object_namespace_addr = format!(
        "https://api.cloudflare.com/client/v4/accounts/{}/workers/durable_objects/namespaces/{}",
        account_id, namespace_id
    );

    let client = http::legacy_auth_client(user);
    let res = client
        .put(&durable_object_namespace_addr)
        .header("Content-Type", "application/json")
        .body(serde_json::to_string(body)?)
        .send()?;

    if !res.status().is_success() {
        failure::bail!(
            "Failed to update a Durable Object namespace! Status: {}, Details {}, Body: {:?}",
            res.status(),
            res.text()?,
            body
        )
    }

    Ok(())
}

#[derive(Serialize, Clone, Deserialize, Debug, PartialEq)]
struct ApiDurableObjectNSResponse {
    pub name: String,
    pub id: String,
    pub script: Option<String>,
    pub class: Option<String>,
}

#[derive(Deserialize)]
struct ApiSingleDurableObjectNSResponse {
    pub result: Option<ApiDurableObjectNSResponse>,
}

#[derive(Deserialize)]
struct ApiListDurableObjectNSResponse {
    pub result: Option<Vec<ApiDurableObjectNSResponse>>,
}

fn list_namespaces_by_name_to_id(
    client: &Client,
    account_id: &str,
) -> Result<HashMap<String, ApiDurableObjectNSResponse>, failure::Error> {
    let mut map = HashMap::new();
    let namespaces = list_namespaces(client, account_id)?;
    for namespace in namespaces {
        map.insert(namespace.name.clone(), namespace);
    }

    Ok(map)
}

fn list_namespaces(
    client: &Client,
    account_id: &str,
) -> Result<Vec<ApiDurableObjectNSResponse>, failure::Error> {
    let list_addr = format!(
        "https://api.cloudflare.com/client/v4/accounts/{}/workers/durable_objects/namespaces",
        account_id,
    );

    let res = client
        .get(&list_addr)
        .header("Content-type", "application/json")
        .send()?;

    if !res.status().is_success() {
        failure::bail!(
            "Failed to list Durable Object namespaces! Status: {}, Details {}",
            res.status(),
            res.text()?
        )
    }

    Ok(res
        .json::<ApiListDurableObjectNSResponse>()?
        .result
        .unwrap_or_default())
}
