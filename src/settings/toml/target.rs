use super::durable_objects::DurableObjects;
use super::kv_namespace::KvNamespace;
pub(crate) use super::manifest::LazyAccountId;
use super::r2_bucket::R2Bucket;
use super::site::Site;
use super::target_type::TargetType;
use super::UsageModel;
use super::{builder::Builder, migrations::Migrations};

use std::collections::HashMap;
use std::env;

use std::path::PathBuf;

#[derive(Clone, Debug, Default)]
pub struct Target {
    pub account_id: LazyAccountId,
    pub kv_namespaces: Vec<KvNamespace>,
    pub r2_buckets: Vec<R2Bucket>,
    pub durable_objects: Option<DurableObjects>,
    pub migrations: Option<Migrations>,
    pub name: String,
    pub target_type: TargetType,
    pub webpack_config: Option<String>,
    pub build: Option<Builder>,
    pub site: Option<Site>,
    pub vars: Option<HashMap<String, String>>,
    pub text_blobs: Option<HashMap<String, PathBuf>>,
    pub usage_model: Option<UsageModel>,
    pub wasm_modules: Option<HashMap<String, PathBuf>>,
    pub compatibility_date: Option<String>,
    pub compatibility_flags: Vec<String>,
}

impl Target {
    pub fn add_kv_namespace(&mut self, kv_namespace: KvNamespace) {
        self.kv_namespaces.push(kv_namespace);
    }

    pub fn package_dir(&self) -> Result<PathBuf, std::io::Error> {
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
