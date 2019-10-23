use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::terminal::message;

use super::{Environment, KvNamespace, Site, TargetType};

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct TemplateConfig {
    pub account_id: Option<String>,
    pub env: Option<HashMap<String, Environment>>,
    #[serde(rename = "kv-namespaces")]
    pub kv_namespaces: Option<Vec<KvNamespace>>,
    #[serde(rename = "type")]
    pub target_type: TargetType,
    pub route: Option<String>,
    pub routes: Option<HashMap<String, String>>,
    pub webpack_config: Option<String>,
    pub workers_dev: Option<bool>,
    pub zone_id: Option<String>,
    pub site: Option<Site>,
}

impl TemplateConfig {
    pub fn warn_on_account_info(&self) {
        let mut top_level_fields: Vec<String> = Vec::new();
        if let Some(account_id) = &self.account_id {
            if !account_id.is_empty() {
                top_level_fields.push("account_id".to_string());
            }
        }
        if let Some(kv_namespaces) = &self.kv_namespaces {
            for kv_namespace in kv_namespaces {
                if !kv_namespace.id.is_empty() && !kv_namespace.binding.is_empty() {
                    top_level_fields.push(format!("kv-namespace {}", kv_namespace.binding));
                }
            }
        }
        if let Some(route) = &self.route {
            if !route.is_empty() {
                top_level_fields.push("route".to_string());
            }
        }
        if let Some(zone_id) = &self.zone_id {
            if !zone_id.is_empty() {
                top_level_fields.push("zone_id".to_string());
            }
        }

        let mut env_fields: HashMap<String, Vec<String>> = HashMap::new();

        if let Some(env) = &self.env {
            for (env_name, env) in env {
                let mut current_env_fields: Vec<String> = Vec::new();
                if let Some(account_id) = &env.account_id {
                    if !account_id.is_empty() {
                        current_env_fields.push("account_id".to_string());
                    }
                }
                if let Some(kv_namespaces) = &env.kv_namespaces {
                    for kv_namespace in kv_namespaces {
                        if !kv_namespace.id.is_empty() && !kv_namespace.binding.is_empty() {
                            current_env_fields
                                .push(format!("kv-namespace {}", kv_namespace.binding));
                        }
                    }
                }
                if let Some(route) = &env.route {
                    if !route.is_empty() {
                        current_env_fields.push("route".to_string());
                    }
                }
                if let Some(zone_id) = &env.zone_id {
                    if !zone_id.is_empty() {
                        current_env_fields.push("zone_id".to_string());
                    }
                }
                if !current_env_fields.is_empty() {
                    env_fields.insert(env_name.to_string(), current_env_fields);
                }
            }
        }
        let top_level_separator = "\n- ";
        let env_separator = "\n  - ";
        if !top_level_fields.is_empty() || !env_fields.is_empty() {
            message::warn("Replace all account specific info in your wrangler.toml.");
            message::warn(
                "Your zone_id and account_id can be found in the right sidebar at https://dash.cloudflare.com",
            );
            if !top_level_fields.is_empty() {
                let top_level_fields = top_level_fields
                    .clone()
                    .into_iter()
                    .collect::<Vec<String>>()
                    .join(top_level_separator);
                println!("{}{}", top_level_separator, top_level_fields);
            }
            if !env_fields.is_empty() {
                for (env_name, env_fields) in env_fields {
                    let msg_prefix = format!("[env.{}]", env_name);
                    let env_fields = env_fields
                        .clone()
                        .into_iter()
                        .collect::<Vec<String>>()
                        .join(env_separator);
                    println!("{}{}{}", msg_prefix, env_separator, env_fields);
                }
            }
        }
    }
}
