pub mod kv_namespace;
mod project_type;

pub use kv_namespace::KvNamespace;
pub use project_type::ProjectType;

use crate::terminal::emoji;
use crate::terminal::message;

use std::collections::{HashMap, HashSet};
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
    pub private: Option<bool>,
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

        let manifest: Manifest = config.try_into()?;

        check_for_duplicate_names(&manifest)?;

        Ok(manifest)
    }

    fn get_environment(
        &self,
        environment_name: Option<&str>,
    ) -> Result<Option<&Environment>, failure::Error> {
        if let Some(environment_name) = environment_name {
            if let Some(environment_table) = &self.env {
                if let Some(environment) = environment_table.get(environment_name) {
                    Ok(Some(environment))
                } else {
                    failure::bail!(format!(
                        "{} Could not find environment with name {}",
                        emoji::WARN,
                        environment_name
                    ))
                }
            } else {
                failure::bail!(format!(
                    "{} There are no environments specified in your wrangler.toml",
                    emoji::WARN
                ))
            }
        } else {
            Ok(None)
        }
    }

    // TODO: when --release is deprecated, this will be much easier
    fn zoneless_or_dot_dev(
        &self,
        environment: Option<&Environment>,
        release: bool,
    ) -> Result<(Option<String>, bool), failure::Error> {
        let use_dot_dev_warning =
            "Please specify the workers_dot_dev boolean in the top level of your wrangler.toml";
        let wdd_failure = format!(
            "{} Your environment should only include `workers_dot_dev` or `route`. If you are trying to publish to workers.dev, add `workers_dot_dev = true`, if you are trying to publish to your own domain, add a route.",
            emoji::WARN
        );

        // TODO: deprecate --release, remove warnings and parsing
        // switch wrangler publish behavior to act the same at top level
        // and environments
        // brace yourself, this is hairy
        let workers_dot_dev: bool = match environment {
            // top level configuration
            None => {
                if release {
                    // --release means zoned, not workers.dev
                    match self.workers_dot_dev {
                        Some(_) => failure::bail!(use_dot_dev_warning),
                        None => {
                            message::warn(use_dot_dev_warning);
                            false // workers_dot_dev defaults to false when it's top level and --release is passed
                        }
                    }
                } else {
                    if let Some(wdd) = self.workers_dot_dev {
                        if wdd {
                            if let Some(route) = &self.route {
                                if !route.is_empty() {
                                    failure::bail!(wdd_failure)
                                }
                            }
                        }
                        wdd
                    } else {
                        message::warn(use_dot_dev_warning);
                        true
                    }
                }
            }

            // environment configuration
            Some(environment) => {
                if let Some(wdd) = environment.workers_dot_dev {
                    if wdd && environment.route.is_some() {
                        failure::bail!(wdd_failure)
                    }
                    wdd
                } else {
                    if let Some(wdd) = self.workers_dot_dev {
                        if wdd && environment.route.is_some() {
                            false // allow route to override workers_dot_dev = true if wdd is inherited
                        } else {
                            wdd // inherit from top level
                        }
                    } else {
                        false // if absent -> false
                    }
                }
            }
        };

        let route = if let Some(environment) = environment {
            if let Some(route) = &environment.route {
                if let Some(wdd) = environment.workers_dot_dev {
                    if wdd {
                        failure::bail!(wdd_failure);
                    }
                }
                Some(route.clone())
            } else {
                None
            }
        } else {
            self.route.clone()
        };

        Ok((route, workers_dot_dev))
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
            message::warn("--release will be deprecated");
        }

        let environment = self.get_environment(environment_name)?;
        let (route, workers_dot_dev) = self.zoneless_or_dot_dev(environment, release)?;
        let deprecate_private_warning = "The 'private' field is now considered deprecated; please use \
        workers_dot_dev to toggle between publishing to your workers.dev subdomain and your own domain.";

        // Check for the presence of the 'private' field in top-level config; if present, warn.
        if self.private.is_some() {
            message::warn(deprecate_private_warning);
        }

        // Also check for presence of 'private' field in a provided environment; if present, warn
        if let Some(e) = environment {
            if e.private.is_some() {
                message::warn(deprecate_private_warning);
            }
        }

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

        let name = if let Some(environment) = environment {
            if let Some(name) = &environment.name {
                let name = name.clone();
                if name == self.name {
                    failure::bail!(format!(
                        "{} Each `name` in your wrangler.toml must be unique",
                        emoji::WARN
                    ))
                }
                name
            } else {
                match environment_name {
                    Some(environment_name) => format!("{}-{}", self.name, environment_name),
                    None => failure::bail!("You must specify `name` in your wrangler.toml"),
                }
            }
        } else {
            self.name.clone()
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
            workers_dot_dev, // MAY inherit,
            // importantly, the top level name will be modified
            // to include the name of the environment
            name,          // MAY inherit
            kv_namespaces, // MUST NOT inherit
            route,         // MUST NOT inherit
            routes,        // MUST NOT inherit
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

fn check_for_duplicate_names(manifest: &Manifest) -> Result<(), failure::Error> {
    let mut names: HashSet<String> = HashSet::new();
    let mut duplicate_names: HashSet<String> = HashSet::new();
    names.insert(manifest.name.to_string());
    if let Some(environments) = &manifest.env {
        for (_, environment) in environments.iter() {
            if let Some(name) = &environment.name {
                if names.contains(name) && !duplicate_names.contains(name) {
                    duplicate_names.insert(name.to_string());
                } else {
                    names.insert(name.to_string());
                }
            }
        }
    }
    let duplicate_name_string = duplicate_names
        .clone()
        .into_iter()
        .collect::<Vec<String>>()
        .join(", ");
    let duplicate_message = match duplicate_names.len() {
        1 => Some("this name is duplicated".to_string()),
        n if n >= 2 => Some("these names are duplicated".to_string()),
        _ => None,
    };
    if let Some(message) = duplicate_message {
        failure::bail!(format!(
            "{} Each name in your `wrangler.toml` must be unique, {}: {}",
            emoji::WARN,
            message,
            duplicate_name_string
        ))
    }
    Ok(())
}

#[cfg(test)]
mod tests;
