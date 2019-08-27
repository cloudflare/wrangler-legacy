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
        let config_path = Path::new("./wrangler.toml");
        get_project_config(None, config_path)
    }

    pub fn new_from_environment(environment: &str) -> Result<Self, failure::Error> {
        let config_path = Path::new("./wrangler.toml");
        get_project_config(Some(environment), config_path)
    }

    pub fn kv_namespaces(&self) -> Vec<KvNamespace> {
        self.kv_namespaces.clone().unwrap_or_else(Vec::new)
    }

    pub fn get_default_environment(
        command_name: &str,
        config_path: &Path,
    ) -> Result<Option<String>, failure::Error> {
        let s = read_config(config_path)?;

        let defaults = s.get_table("defaults")?;

        let default = defaults.get(command_name);

        if default.is_none() {
            failure::bail!(format!(
                "{} There is no default environment specified for {}",
                emoji::WARN,
                command_name
            ))
        }
        Ok(Some(default.unwrap().to_string()))
    }
}

fn read_config(config_path: &Path) -> Result<Config, failure::Error> {
    let mut s = Config::new();

    let config_str = config_path
        .to_str()
        .expect("project config path should be a string");
    s.merge(File::with_name(config_str))?;

    // Eg.. `CF_ACCOUNT_AUTH_KEY=farts` would set the `account_auth_key` key
    s.merge(Environment::with_prefix("CF"))?;
    Ok(s)
}

fn get_project_config(
    environment_name: Option<&str>,
    config_path: &Path,
) -> Result<Project, failure::Error> {
    let s = read_config(config_path)?;

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

    let environments = s.get_table("env");
    if environments.is_err() {
        message::warn("Your `wrangler.toml` is outdated, see <link> for an example configuration.");
        let project: Result<Project, config::ConfigError> = s.try_into();
        return project.map_err(|e| {
            let msg = format!(
                "{} Your project config has an error, check your `wrangler.toml`: {}",
                emoji::WARN,
                e
            );
            failure::err_msg(msg)
        });
    }
    if environment_name.is_none() {
        failure::bail!(
            r##"
You either need to specify an environment with --environment or specify default environments like so:
[defaults]
publish = "staging"
preview = "production"
"##
        )
    }
    let environments = environments?;
    let environment_name = environment_name.unwrap();
    let environment = match environments.get(environment_name) {
        Some(e) => e,
        None => failure::bail!(format!(
            "{0} Your `wrangler.toml` does not contain a `{1}` environment {0}",
            emoji::WARN,
            environment_name
        )),
    };

    let project = environment.clone().try_into()?;
    Ok(project)
}

#[cfg(test)]
mod tests;
