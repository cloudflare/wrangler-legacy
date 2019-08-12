pub mod kv_namespace;
mod project_type;

pub use kv_namespace::KvNamespace;
pub use project_type::ProjectType;

use crate::terminal::emoji;
use crate::terminal::message;

use std::collections::HashMap;
use std::convert::TryFrom;
use std::fs;
use std::path::{Path, PathBuf};
use std::str::FromStr;

use log::info;

use config::{Config, Environment, File, Value};
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

    pub fn new(environment: Option<&str>) -> Result<Self, failure::Error> {
        let config_path = Path::new("./wrangler.toml");

        get_project_config(environment, config_path)
    }

    pub fn kv_namespaces(&self) -> Vec<KvNamespace> {
        self.kv_namespaces.clone().unwrap_or_else(Vec::new)
    }
}

impl TryFrom<HashMap<String, config::Value>> for Project {
    type Error = String;

    fn try_from(map: HashMap<String, config::Value>) -> Result<Self, Self::Error> {
        let project_type = map
            .get("type")
            .ok_or("Environment does not have a `type`".to_string())?;
        let project_type = project_type.clone().into_str().map_err(|e| e.to_string())?;
        let project_type = ProjectType::from_str(&project_type).map_err(|e| e.to_string())?;
        let name = map
            .get("name")
            .ok_or("Environment does not have a `name`")?;
        let name = name.clone().into_str().map_err(|e| e.to_string())?;
        let account_id = map
            .get("account_id")
            .ok_or("Environment does not have an `account_id`")?;
        let account_id = account_id.clone().into_str().map_err(|e| e.to_string())?;
        let private = map
            .get("private")
            .map(|p| p.clone().into_bool().map_err(|e| e.to_string()))
            .transpose()?;
        let zone_id = map
            .get("zone_id")
            .map(|z| z.clone().into_str().map_err(|e| e.to_string()))
            .transpose()?;
        let route = map
            .get("route")
            .map(|r| r.clone().into_str().map_err(|e| e.to_string()))
            .transpose()?;
        println!("{:#?}", map.get("routes"));
        let routes = map
            .get("routes")
            .map(|r| r.clone().into_table().map_err(|e| e.to_string()))
            .transpose()?;
        // if routes.is_some() {
        //     let routes_map: HashMap<String, String> = routes.map(|r| {
        //         let new_routes = HashMap::new();
        //         for (k, v) in r.iter() {
        //             let value = v.clone().into_str().map_err(|e| e.to_string());
        //             new_routes.insert(k, value);
        //         }
        //         new_routes
        //     });
        //     let routes = Some(routes_map);
        // }
        // println!("{:#?}", routes);
        let kv_namespaces = map
            .get("kv_namespaces")
            .map(|k| k.clone().into_array().map_err(|e| e.to_string()))
            .transpose()?;
        let webpack_config = map
            .get("webpack_config")
            .map(|w| w.clone().into_str().map_err(|e| e.to_string()))
            .transpose()?;
        Ok(Project {
            name,
            project_type,
            private,
            zone_id,
            account_id,
            route,
            routes: None,        // TODO implement
            kv_namespaces: None, // TODO implement
            webpack_config,
        })
    }
}

fn get_project_config(
    environment_name: Option<&str>,
    config_path: &Path,
) -> Result<Project, failure::Error> {
    let mut s = Config::new();

    let config_str = config_path
        .to_str()
        .expect("project config path should be a string");
    s.merge(File::with_name(config_str))?;

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

    let project_type: Result<String, config::ConfigError> = s.get("type");
    if project_type.is_err() {
        failure::bail!(format!(
            "{0} Your `wrangler.toml` is missing a `type` field {0}",
            emoji::WARN
        ))
    }
    let project_type = project_type.unwrap();

    let environments = s.get_table("env");
    if environments.is_err() {
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
    let environments = environments.unwrap();
    let environment_name = match environment_name {
        None => "default",
        Some(x) => x,
    };
    let environment = match environments.get(environment_name) {
        Some(e) => e,
        None => failure::bail!(format!(
            "{0} Your `wrangler.toml` does not contain a `{1}` environment {0}",
            emoji::WARN,
            environment_name
        )),
    };
    let env_parse_err = format!(
        "{0} Your `{1}` environment could not be parsed {0}",
        emoji::WARN,
        environment_name
    );
    let environment_table = environment.clone().into_table();
    if environment_table.is_err() {
        failure::bail!(env_parse_err)
    }
    let mut environment_table = environment_table.unwrap();
    environment_table.insert("type".to_string(), Value::new(None, project_type));
    let project = Project::try_from(environment_table).map_err(|e| failure::err_msg(e))?;
    Ok(project)
}

#[cfg(test)]
mod tests;
