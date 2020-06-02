use super::kv_namespace::{ConfigKvNamespace, KvNamespace};
use super::site::Site;
use super::target_type::TargetType;
use crate::terminal::message;

use std::collections::HashMap;
use std::env;

use std::path::PathBuf;

use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct Target {
    pub account_id: String,
    #[serde(rename = "kv-namespaces")]
    pub kv_namespaces: Option<Vec<ConfigKvNamespace>>,
    pub name: String,
    #[serde(rename = "type")]
    pub target_type: TargetType,
    pub webpack_config: Option<String>,
    pub site: Option<Site>,
    pub vars: Option<HashMap<String, String>>,
}

impl Target {
    pub fn kv_namespaces(&self) -> Vec<KvNamespace> {
        if let Some(kv_namespaces) = &self.kv_namespaces {
            let mut parsed_namespaces = Vec::new();
            for kv_namespace in kv_namespaces {
                parsed_namespaces.push(KvNamespace {
                    id: kv_namespace.id.to_string(),
                    binding: kv_namespace.binding.to_string(),
                });
            }
            parsed_namespaces
        } else {
            Vec::new()
        }
    }

    pub fn preview_kv_namespaces(&self) -> Result<Vec<KvNamespace>, failure::Error> {
        if let Some(kv_namespaces) = &self.kv_namespaces {
            let mut parsed_namespaces = Vec::new();
            for kv_namespace in kv_namespaces {
                if let Some(preview_id) = &kv_namespace.preview_id {
                    if preview_id == &kv_namespace.id {
                        message::warn("Specifying the same KV namespace ID for both preview and production sessions may cause bugs in your production worker! Proceed with caution.");
                    }
                    parsed_namespaces.push(KvNamespace {
                        id: preview_id.to_string(),
                        binding: kv_namespace.binding.to_string(),
                    });
                } else {
                    failure::bail!("In order to preview a worker with KV namespaces, you must designate a preview_id for each KV namespace you'd like to preview.")
                }
            }
            Ok(parsed_namespaces)
        } else {
            Ok(Vec::new())
        }
    }

    pub fn add_kv_namespace(&mut self, kv_namespace: ConfigKvNamespace) {
        if let Some(kv_namespaces) = &mut self.kv_namespaces {
            kv_namespaces.push(kv_namespace);
        } else {
            self.kv_namespaces = Some(vec![kv_namespace]);
        }
    }

    pub fn build_dir(&self) -> Result<PathBuf, std::io::Error> {
        // if `site` is configured, we want to isolate worker code
        // and build artifacts away from static site application code.
        match &self.site {
            Some(site_config) => site_config.entry_point(),
            None => {
                let current_dir = env::current_dir()?;
                Ok(current_dir)
            }
        }
    }
}
