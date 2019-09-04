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
    pub env: Option<HashMap<String, Environment>>,
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
    pub fn new(config_path: &Path) -> Result<Self, failure::Error> {
        let config = read_config(config_path)?;

        // check for pre 1.1.0 KV namespace format
        let kv_namespaces: Result<Vec<config::Value>, config::ConfigError> =
            config.get("kv-namespaces");

        validate_kv_namespaces_config(kv_namespaces)?;

        let manifest = config.try_into()?;
        Ok(manifest)
    }

    pub fn get_target(
        &self,
        environment_name: Option<&str>,
        release: bool,
    ) -> Result<Target, failure::Error> {
        if release && self.workers_dot_dev.is_some() {
            failure::bail!(
                "The --release flag is not compatible with use of the workers_dot_dev field"
            )
        }

        if release {
            message::warn("--release will be deprecated, please use --env or specify workers_dot_dev in your `wrangler.toml`");
        }

        let environment = match environment_name {
            Some(environment_name) => match &self.env {
                Some(environment_table) => {
                    let environment = environment_table.get(environment_name);
                    match environment {
                        Some(environment) => Some(environment),
                        None => failure::bail!(format!(
                            "{} Could not find environment with name {}",
                            emoji::WARN,
                            environment_name
                        )),
                    }
                }
                None => failure::bail!(format!(
                    "{} There are no environments specified in your wrangler.toml",
                    emoji::WARN
                )),
            },
            None => None,
        };

        let deprecate_warning = "please specify workers_dot_dev in your wrangler.toml";

        let workers_dot_dev = match environment {
            // environment specified
            Some(environment) => match environment.workers_dot_dev {
                Some(wdd) => wdd,
                None => false,
            },
            // top level (legacy)
            None => {
                if release {
                    message::warn(deprecate_warning);
                    false // --release means not workers.dev
                } else {
                    match self.workers_dot_dev {
                        Some(wdd) => wdd,
                        None => {
                            message::warn(deprecate_warning);
                            true // no --release means workers.dev
                        }
                    }
                }
            }
        };

        let kv_namespaces = match environment {
            Some(environment) => match &environment.kv_namespaces {
                Some(kv) => Some(kv.clone()),
                None => None,
            },
            None => self.kv_namespaces.clone(),
        };

        let account_id = match environment {
            Some(environment) => match &environment.account_id {
                Some(a) => a.clone(),
                None => self.account_id.clone(),
            },
            None => self.account_id.clone(),
        };

        let name = match environment {
            Some(environment) => match &environment.name {
                Some(name) => {
                    let name = name.clone();
                    if name == self.name {
                        failure::bail!("Each `name` in your wrangler.toml must be unique")
                    }
                    name
                }
                None => failure::bail!("You must specify `name` in your wrangler.toml"),
            },
            None => self.name.clone(),
        };

        let route = match environment {
            Some(environment) => match &environment.route {
                Some(route) => Some(route.clone()),
                None => None,
            },
            None => self.route.clone(),
        };

        let routes = match environment {
            Some(environment) => match &environment.routes {
                Some(routes) => Some(routes.clone()),
                None => None,
            },
            None => self.routes.clone(),
        };

        let webpack_config = match environment {
            Some(environment) => match &environment.webpack_config {
                Some(webpack_config) => Some(webpack_config.clone()),
                None => self.webpack_config.clone(),
            },
            None => self.webpack_config.clone(),
        };

        let zone_id = match environment {
            Some(environment) => match &environment.zone_id {
                Some(zone_id) => Some(zone_id.clone()),
                None => self.zone_id.clone(),
            },
            None => self.zone_id.clone(),
        };

        let project_type = self.project_type.clone();

        Ok(Target {
            project_type,    // MUST inherit
            account_id,      // MAY inherit
            webpack_config,  // MAY inherit
            zone_id,         // MAY inherit
            kv_namespaces,   // MUST NOT inherit
            name,            // MUST NOT inherit
            route,           // MUST NOT inherit
            routes,          // MUST NOT inherit
            workers_dot_dev, // MUST NOT inherit,
        })
    }

    pub fn generate(
        name: String,
        project_type: ProjectType,
        init: bool,
    ) -> Result<Manifest, failure::Error> {
        let manifest = Manifest {
            account_id: String::new(),
            env: None,
            kv_namespaces: None,
            name: name.clone(),
            private: None,
            project_type: project_type.clone(),
            route: Some(String::new()),
            routes: None,
            webpack_config: None,
            workers_dot_dev: Some(true),
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

#[cfg(test)]
mod tests;
