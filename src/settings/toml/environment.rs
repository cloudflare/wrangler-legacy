use serde::{Deserialize, Serialize};

use crate::settings::toml::deploy_target::RouteConfig;
use crate::settings::toml::kv_namespace::KvNamespace;
use crate::settings::toml::site::Site;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Environment {
    pub name: Option<String>,
    pub account_id: Option<String>,
    pub workers_dev: Option<bool>,
    pub route: Option<String>,
    pub routes: Option<Vec<String>>,
    pub zone_id: Option<String>,
    pub webpack_config: Option<String>,
    pub private: Option<bool>,
    pub site: Option<Site>,
    #[serde(rename = "kv-namespaces")]
    pub kv_namespaces: Option<Vec<KvNamespace>>,
}

impl Environment {
    pub fn route_config(
        &self,
        top_level_account_id: String,
        top_level_zone_id: Option<String>,
    ) -> Option<RouteConfig> {
        // TODO: Deserialize empty strings to None
        let account_id = if empty(&self.account_id) {
            Some(top_level_account_id)
        } else {
            self.account_id.clone()
        };

        // TODO: Deserialize empty strings to None
        let zone_id = if empty(&self.zone_id) {
            top_level_zone_id
        } else {
            self.zone_id.clone()
        };

        if self.workers_dev.is_none() && self.route.is_none() && self.routes.is_none() {
            None
        } else {
            Some(RouteConfig {
                account_id: account_id,
                workers_dev: self.workers_dev,
                route: self.route.clone(),
                routes: self.routes.clone(),
                zone_id: zone_id,
            })
        }
    }
}

// TODO: Deserialize empty strings to None
fn empty(optional_string: &Option<String>) -> bool {
    if let Some(string) = optional_string {
        string.is_empty()
    } else {
        true
    }
}
