use super::kv_namespace::KvNamespace;
use super::site::Site;
use super::target_type::TargetType;
use super::Route;

use std::env;

use std::path::PathBuf;

use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Target {
    pub account_id: String,
    #[serde(rename = "kv-namespaces")]
    pub kv_namespaces: Option<Vec<KvNamespace>>,
    pub name: String,
    #[serde(rename = "type")]
    pub target_type: TargetType,
    pub route: Option<String>,
    pub routes: Option<Vec<String>>,
    pub webpack_config: Option<String>,
    pub zone_id: Option<String>,
    pub site: Option<Site>,
}

impl Target {
    pub fn kv_namespaces(&self) -> Vec<KvNamespace> {
        self.kv_namespaces.clone().unwrap_or_else(Vec::new)
    }

    pub fn add_kv_namespace(&mut self, kv_namespace: KvNamespace) {
        let mut updated_namespaces = self.kv_namespaces();
        updated_namespaces.push(kv_namespace);
        self.kv_namespaces = Some(updated_namespaces);
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

    pub fn routes(&self) -> Result<Vec<Route>, failure::Error> {
        let mut routes = Vec::new();

        // we should assert that only one of the two keys is specified in the user's toml.
        if self.route.is_some() && self.routes.is_some() {
            failure::bail!("You can specify EITHER `route` or `routes` in your wrangler.toml");
        }

        // everything outside of this module should consider `target.routes()` to be a Vec;
        // the fact that you can specify singular or plural is a detail of the wrangler.toml contract.
        if let Some(single_route) = &self.route {
            routes.push(Route {
                script: Some(self.name.to_owned()),
                pattern: single_route.to_string(),
            });
        } else if let Some(multi_route) = &self.routes {
            for pattern in multi_route {
                routes.push(Route {
                    script: Some(self.name.to_owned()),
                    pattern: pattern.to_string(),
                });
            }
        }

        Ok(routes)
    }
}
