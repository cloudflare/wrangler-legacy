use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use serde_with::rust::string_empty_as_none;

use crate::settings::toml::deploy_config::RouteConfig;
use crate::settings::toml::kv_namespace::KvNamespace;
use crate::settings::toml::site::Site;

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct Environment {
    pub name: Option<String>,
    #[serde(default, with = "string_empty_as_none")]
    pub account_id: Option<String>,
    pub workers_dev: Option<bool>,
    #[serde(default, with = "string_empty_as_none")]
    pub route: Option<String>,
    pub routes: Option<Vec<String>>,
    #[serde(default, with = "string_empty_as_none")]
    pub zone_id: Option<String>,
    pub webpack_config: Option<String>,
    pub private: Option<bool>,
    pub site: Option<Site>,
    #[serde(rename = "kv-namespaces")]
    pub kv_namespaces: Option<Vec<KvNamespace>>,
    pub config: Option<HashMap<String, String>>,
}

impl Environment {
    pub fn route_config(
        &self,
        top_level_account_id: String,
        top_level_zone_id: Option<String>,
    ) -> Option<RouteConfig> {
        let account_id = if self.account_id.is_none() {
            Some(top_level_account_id)
        } else {
            self.account_id.clone()
        };

        let zone_id = if self.zone_id.is_none() {
            top_level_zone_id
        } else {
            self.zone_id.clone()
        };

        if self.workers_dev.is_none() && self.route.is_none() && self.routes.is_none() {
            None
        } else {
            Some(RouteConfig {
                account_id,
                workers_dev: self.workers_dev,
                route: self.route.clone(),
                routes: self.routes.clone(),
                zone_id,
            })
        }
    }
}
