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
    pub fn route_config(&self) -> Result<RouteConfig, failure::Error> {
        Ok(RouteConfig {
            workers_dev: self.workers_dev,
            route: self.route.clone(),
            routes: self.routes.clone(),
            zone_id: self.zone_id.clone(),
        })
    }
}
