pub mod kv_namespace;
mod project_type;

pub use kv_namespace::KvNamespace;
pub use project_type::ProjectType;

use crate::terminal::emoji;
use crate::terminal::message;

use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

use log::info;

use config::{Config, File};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Target {
    pub account_id: String,
    #[serde(rename = "kv-namespaces")]
    pub kv_namespaces: Option<Vec<KvNamespace>>,
    pub name: String,
    #[serde(rename = "type")]
    pub project_type: ProjectType,
    pub route: Option<String>,
    pub routes: Option<HashMap<String, String>>,
    pub webpack_config: Option<String>,
    pub workers_dot_dev: bool,
    pub zone_id: Option<String>,
}

impl Target {
    pub fn kv_namespaces(&self) -> Vec<KvNamespace> {
        self.kv_namespaces.clone().unwrap_or_else(Vec::new)
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Environment {
    pub account_id: Option<String>,
    #[serde(rename = "kv-namespaces")]
    pub kv_namespaces: Option<Vec<KvNamespace>>,
    pub name: Option<String>,
    pub route: Option<String>,
    pub routes: Option<HashMap<String, String>>,
    pub webpack_config: Option<String>,
    pub workers_dot_dev: Option<bool>,
    pub zone_id: Option<String>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Manifest {
    pub account_id: String,
    pub environments: HashMap<String, Environment>,
    #[serde(rename = "kv-namespaces")]
    pub kv_namespaces: Option<Vec<KvNamespace>>,
    pub name: String,
    pub private: Option<bool>,
    #[serde(rename = "type")]
    pub project_type: ProjectType,
    pub route: Option<String>,
    pub routes: Option<HashMap<String, String>>,
    pub webpack_config: Option<String>,
    pub workers_dot_dev: Option<bool>,
    pub zone_id: Option<String>,
}

impl Manifest {
    pub fn new() -> Result<Self, failure::Error> {
        get_manifest(Path::new("./wrangler.toml"))
    }

    pub fn get_target(
        &self,
        environment_name: Option<&str>,
        release: bool,
    ) -> Result<Target, failure::Error> {
        if release && self.workers_dot_dev.is_some() {
            failure::bail!("The --release flag is deprecated with use of the workers_dot_dev field")
        }
        let environment = if environment_name.is_none() {
            None
        } else {
            let environment_name = environment_name.unwrap();
            let environment = self.environments.get(environment_name);
            if environment.is_none() {
                failure::bail!(format!(
                    "{} Could not find environment with name {}",
                    emoji::WARN,
                    environment_name
                ))
            }
            Some(environment.unwrap())
        };
        let workers_dot_dev = if environment.is_none() {
            // wrangler publish --release
            if release {
                // not workers.dev
                false
            // wrangler publish
            } else {
                if self.workers_dot_dev.is_none() {
                    // workers.dev
                    true
                } else {
                    // use .toml value
                    self.workers_dot_dev.unwrap()
                }
            }
        } else {
            // wrangler publish --env foo
            let environment = environment.unwrap();
            if environment.workers_dot_dev.is_none() {
                // not workers.dev
                false
            } else {
                // use .toml value
                environment.workers_dot_dev.unwrap()
            }
        };

        Ok(Target {
            account_id: self.account_id,
            kv_namespaces: self.kv_namespaces,
            name: self.name,
            project_type: self.project_type,
            route: self.route,
            routes: self.routes,
            webpack_config: self.webpack_config,
            workers_dot_dev,
            zone_id: self.zone_id,
        })
    }

    pub fn generate(
        name: String,
        project_type: ProjectType,
        init: bool,
    ) -> Result<Manifest, failure::Error> {
        let manifest = Manifest {
            account_id: String::new(),
            environments: HashMap::new(),
            kv_namespaces: None,
            name: name.clone(),
            private: None,
            project_type: project_type.clone(),
            route: Some(String::new()),
            routes: None,
            webpack_config: None,
            workers_dot_dev: Some(false),
            zone_id: Some(String::new()),
        };

        let toml = toml::to_string(&manifest)?;
        let config_path = if init {
            PathBuf::from("./")
        } else {
            Path::new("./").join(&name)
        };
        let config_file = config_path.join("wrangler.toml");

        info!("Writing a wrangler.toml file at {}", config_file.display());
        fs::write(&config_file, &toml)?;
        Ok(manifest)
    }
}

fn read_config(config_path: &Path) -> Result<Config, failure::Error> {
    let mut config = Config::new();

    let config_str = config_path
        .to_str()
        .expect("project config path should be a string");
    config.merge(File::with_name(config_str))?;

    // Eg.. `CF_ACCOUNT_AUTH_KEY=farts` would set the `account_auth_key` key
    config.merge(config::Environment::with_prefix("CF"))?;

    Ok(config)
}

fn validate_kv_namespaces_config(
    kv_namespaces: Result<Vec<config::Value>, config::ConfigError>,
) -> Result<(), failure::Error> {
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
    Ok(())
}

fn get_manifest(config_path: &Path) -> Result<Manifest, failure::Error> {
    let mut config = read_config(config_path)?;

    // check for pre 1.1.0 KV namespace format
    let kv_namespaces: Result<Vec<config::Value>, config::ConfigError> =
        config.get("kv-namespaces");

    validate_kv_namespaces_config(kv_namespaces)?;

    let manifest = config.try_into()?;
    Ok(manifest)
}

#[cfg(test)]
mod tests;
