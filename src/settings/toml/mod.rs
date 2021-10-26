mod builder;
mod dev;
mod durable_objects;
mod environment;
mod kv_namespace;
mod manifest;
pub mod migrations;
mod r2_bucket;
mod route;
mod site;
pub(crate) mod target;
mod target_type;
mod triggers;

pub use builder::{ModuleRule, UploadFormat};
pub use durable_objects::{DurableObjects, DurableObjectsClass};
pub use kv_namespace::{ConfigKvNamespace, KvNamespace};
pub use manifest::Manifest;
pub use r2_bucket::{ConfigR2Bucket, R2Bucket};
pub use route::{Route, RouteConfig};
pub use site::Site;
pub use target::Target;
pub use target_type::TargetType;

use anyhow::anyhow;
use serde::{Deserialize, Serialize};
use std::str::FromStr;

#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum UsageModel {
    Bundled,
    Unbound,
}

impl FromStr for UsageModel {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "bundled" => Ok(UsageModel::Bundled),
            "unbound" => Ok(UsageModel::Unbound),
            _ => Err(anyhow!(
                "Invalid usage model; must be either \"bundled\" or \"unbound\""
            )),
        }
    }
}

impl AsRef<str> for UsageModel {
    fn as_ref(&self) -> &str {
        match self {
            UsageModel::Bundled => "bundled",
            UsageModel::Unbound => "unbound",
        }
    }
}

#[cfg(test)]
mod tests;
