pub mod kv_namespace;
mod project_type;

pub use kv_namespace::KvNamespace;
pub use project_type::ProjectType;

use crate::terminal::emoji;

use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

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
        let config_path = if init {
            PathBuf::from("./")
        } else {
            Path::new("./").join(&name)
        };
        let config_file = config_path.join("wrangler.toml");

        info!("Writing a wrangler.toml file at {}", config_file.display());
        fs::write(&config_file, &toml)?;
        Ok(project)
    }

    pub fn new() -> Result<Self, failure::Error> {
        let mut config_path = PathBuf::new();
        config_path.push("./wrangler.toml");

        get_project_config(config_path)
    }

    pub fn kv_namespaces(&self) -> Vec<KvNamespace> {
        match &self.kv_namespaces {
            Some(kv) => kv.clone(),
            None => Vec::new(),
        }
    }
}

fn get_project_config(config_path: PathBuf) -> Result<Project, failure::Error> {
    let mut s = Config::new();

    let config_str = config_path
        .to_str()
        .expect("project config path should be a string");
    s.merge(File::with_name(config_str))?;

    // Eg.. `CF_ACCOUNT_AUTH_KEY=farts` would set the `account_auth_key` key
    s.merge(Environment::with_prefix("CF"))?;

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
