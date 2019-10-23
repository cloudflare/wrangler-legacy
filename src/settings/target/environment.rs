use super::kv_namespace::KvNamespace;
use super::site::Site;

use std::collections::HashMap;

use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Environment {
    pub account_id: Option<String>,
    #[serde(rename = "kv-namespaces")]
    pub kv_namespaces: Option<Vec<KvNamespace>>,
    pub name: Option<String>,
    pub private: Option<bool>,
    pub route: Option<String>,
    pub routes: Option<HashMap<String, String>>,
    pub webpack_config: Option<String>,
    pub workers_dev: Option<bool>,
    pub zone_id: Option<String>,
    pub site: Option<Site>,
}
