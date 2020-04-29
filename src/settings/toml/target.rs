use crate::settings::toml::Manifest;

use super::kv_namespace::KvNamespace;
use super::site::Site;
use super::target_type::TargetType;

use std::collections::HashMap;
use std::env;

use std::path::PathBuf;

use serde::{Deserialize, Serialize};

#[derive(Clone, Default, Debug, Deserialize, Serialize)]
pub struct Target {
    pub account_id: String,
    #[serde(rename = "kv-namespaces")]
    // kv_namespaces should never be accessed directly
    // use the method `Target::kv_namespaces()` instead
    kv_namespaces: Option<Vec<KvNamespace>>,
    pub name: String,
    #[serde(rename = "type")]
    pub target_type: TargetType,
    pub webpack_config: Option<String>,
    pub site: Option<Site>,
    pub vars: Option<HashMap<String, String>>,
}

impl Target {
    pub fn from_manifest(
        manifest: &Manifest,
        environment_name: Option<&str>,
    ) -> Result<Self, failure::Error> {
        let target_type = match manifest.site {
            Some(_) => TargetType::Webpack,
            None => manifest.target_type.clone(),
        };

        let mut target = Self {
            target_type,                                     // MUST inherit
            account_id: manifest.account_id.clone(),         // MAY inherit
            webpack_config: manifest.webpack_config.clone(), // MAY inherit
            // importantly, the top level name will be modified
            // to include the name of the environment
            name: manifest.name.clone(),                   // MAY inherit
            kv_namespaces: manifest.kv_namespaces.clone(), // MUST NOT inherit
            site: manifest.site.clone(),                   // MUST NOT inherit
            vars: manifest.vars.clone(),                   // MAY inherit
        };

        let environment = manifest.get_environment(environment_name)?;

        if let Some(environment) = environment {
            target.name = manifest.worker_name(environment_name);
            if let Some(account_id) = &environment.account_id {
                target.account_id = account_id.clone();
            }
            if let Some(webpack_config) = &environment.webpack_config {
                target.webpack_config = Some(webpack_config.clone());
            }

            // don't inherit kv namespaces because it is an anti-pattern
            // to use the same namespaces across multiple environments
            target.kv_namespaces = environment.kv_namespaces.clone();

            // don't inherit vars
            target.vars = environment.vars.clone();
        }

        Ok(target)
    }

    pub fn kv_namespaces(&self) -> Vec<KvNamespace> {
        self.kv_namespaces.clone().unwrap_or_else(Vec::new)
    }

    pub fn add_kv_namespace(&mut self, kv_namespace: KvNamespace) {
        let mut updated_namespaces = self.kv_namespaces();
        updated_namespaces.push(kv_namespace);
        self.kv_namespaces = Some(updated_namespaces);
    }

    pub fn remove_all_kv_namespaces(&mut self) {
        self.kv_namespaces = None;
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_returns_empty_vec() {
        let target = Target::default();
        assert_eq!(target.kv_namespaces().len(), 0)
    }

    #[test]
    fn it_returns_vec_len_1() {
        let target = Target {
            kv_namespaces: Some(vec![KvNamespace {
                id: "012345".to_string(),
                binding: "BINDING".to_string(),
                bucket: None,
            }]),
            ..Default::default()
        };
        assert_eq!(target.kv_namespaces().len(), 1)
    }
}
