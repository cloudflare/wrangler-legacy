pub mod kv_namespace;
mod project_type;

pub use kv_namespace::KvNamespace;
pub use project_type::ProjectType;

use crate::terminal::emoji;
use crate::terminal::message;

use std::collections::HashMap;
use std::fs;
use std::path::Path;

use log::info;

use config::{Config, Environment, File};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Project {
    pub name: String,
    #[serde(rename = "type")]
    pub project_type: ProjectType,
    pub zone_id: Option<String>,
    pub private: Option<bool>,
    pub webpack_config: Option<String>,
    pub account_id: String,
    pub route: Option<String>,
    pub routes: Option<HashMap<String, String>>,
    #[serde(rename = "kv-namespaces")]
    pub kv_namespaces: Option<Vec<KvNamespace>>,
}

impl Project {
    pub fn generate(
        name: String,
        project_type: ProjectType,
        project_dir: &Path,
        init: bool,
    ) -> Result<Project, failure::Error> {
        let project = Project {
            name: name.clone(),
            project_type: project_type.clone(),
            private: Some(false),
            zone_id: Some(String::new()),
            account_id: String::new(),
            route: Some(String::new()),
            routes: None,
            kv_namespaces: None,
            webpack_config: None,
        };

        let toml = toml::to_string(&project)?;
        let init_dir = project_dir.join(&name);
        let config_path = if init { project_dir } else { init_dir.as_ref() };
        let config_file = config_path.join("wrangler.toml");

        info!("Writing a wrangler.toml file at {}", config_file.display());
        fs::write(&config_file, &toml)?;
        Ok(project)
    }

    pub fn new(path: &Path) -> Result<Self, failure::Error> {
        get_project_config(path.join("wrangler.toml").as_ref())
    }

    pub fn kv_namespaces(&self) -> Vec<KvNamespace> {
        self.kv_namespaces.clone().unwrap_or_else(Vec::new)
    }
}

fn get_project_config(config_path: &Path) -> Result<Project, failure::Error> {
    let mut s = Config::new();

    s.merge(File::from(config_path))?;

    // Eg.. `CF_ACCOUNT_AUTH_KEY=farts` would set the `account_auth_key` key
    s.merge(Environment::with_prefix("CF"))?;

    // check for pre 1.1.0 KV namespace format
    let kv_namespaces: Result<Vec<config::Value>, config::ConfigError> = s.get("kv-namespaces");

    if let Ok(values) = kv_namespaces {
        let old_format = values.iter().any(|val| val.clone().into_str().is_ok());

        if old_format {
            message::warn("As of 1.1.0 the kv-namespaces format has been stabilized");
            message::info("Please add a section like this in your `wrangler.toml` for each KV Namespace you wish to bind:");

            let fmt_demo = r##"
[[kv-namespaces]]
binding = "BINDING_NAME"
id = "0f2ac74b498b48028cb68387c421e279"

# binding is the variable name you wish to bind the namespace to in your script.
# id is the namespace_id assigned to your kv namespace upon creation. e.g. (per namespace)
"##;

            println!("{}", fmt_demo);

            let msg = format!("{0} Your project config has an error {0}", emoji::WARN);
            failure::bail!(msg)
        }
    }

    let project: Result<Project, config::ConfigError> = s.try_into();
    match project {
        Ok(s) => Ok(s),
        Err(e) => {
            let msg = format!(
                "{} Your project config has an error, check your `wrangler.toml`: {}",
                emoji::WARN,
                e
            );

            failure::bail!(msg)
        }
    }
}

#[cfg(test)]
mod tests;
