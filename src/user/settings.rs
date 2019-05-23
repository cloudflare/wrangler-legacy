use std::collections::HashMap;
use std::fmt;
use std::fs;
use std::path::Path;

use log::info;

use config::{Config, Environment, File};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct GlobalUserSettings {
    pub email: String,
    pub api_key: String,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ProjectSettings {
    pub name: String,
    #[serde(rename = "type")]
    pub project_type: ProjectType,
    pub zone_id: String,
    pub route: Option<String>,
    pub routes: Option<HashMap<String, String>>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub enum ProjectType {
    JavaScript,
    Rust,
    Webpack,
}

impl fmt::Display for ProjectType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let printable = match *self {
            ProjectType::JavaScript => "js",
            ProjectType::Rust => "rust",
            ProjectType::Webpack => "webpack",
        };
        write!(f, "{}", printable)
    }
}

impl ProjectSettings {
    pub fn generate(
        name: String,
        project_type: ProjectType,
    ) -> Result<ProjectSettings, failure::Error> {
        let project_settings = ProjectSettings {
            name: name.clone(),
            project_type: project_type.clone(),
            zone_id: String::new(),
            route: Some(String::new()),
            routes: None,
        };

        let toml = toml::to_string(&project_settings)?;
        let config_path = Path::new("./").join(&name);
        let config_file = config_path.join("wrangler.toml");

        info!("Writing a wrangler.toml file at {}", config_file.display());
        fs::write(&config_file, &toml)?;
        Ok(project_settings)
    }
}

#[derive(Clone, Serialize)]
pub struct Settings {
    pub global_user: GlobalUserSettings,
    pub project: ProjectSettings,
}

impl Settings {
    pub fn new() -> Result<Self, failure::Error> {
        let global_user = get_global_config()?;
        let project = get_project_config()?;

        Ok(Settings {
            global_user,
            project,
        })
    }
}

fn get_global_config() -> Result<GlobalUserSettings, failure::Error> {
    let mut s = Config::new();

    let config_path = dirs::home_dir()
        .expect("oops no home dir")
        .join(".wrangler/config/default");
    let config_str = config_path
        .to_str()
        .expect("global config path should be a string");
    s.merge(File::with_name(config_str))?;

    // Eg.. `CF_ACCOUNT_AUTH_KEY=farts` would set the `account_auth_key` key
    s.merge(Environment::with_prefix("CF"))?;

    let settings: Result<GlobalUserSettings, config::ConfigError> = s.try_into();
    match settings {
        Ok(s) => Ok(s),
        Err(e) => {
            let msg = format!(
                "⚠️ Your global config has an error, run `wrangler config`: {}",
                e
            );
            Err(failure::err_msg(msg))
        }
    }
}

pub fn get_project_config() -> Result<ProjectSettings, failure::Error> {
    let mut s = Config::new();

    let config_path = Path::new("./wrangler.toml");
    let config_str = config_path
        .to_str()
        .expect("project config path should be a string");
    s.merge(File::with_name(config_str))?;

    // Eg.. `CF_ACCOUNT_AUTH_KEY=farts` would set the `account_auth_key` key
    s.merge(Environment::with_prefix("CF"))?;

    let settings: Result<ProjectSettings, config::ConfigError> = s.try_into();
    match settings {
        Ok(s) => Ok(s),
        Err(e) => {
            let msg = format!(
                "⚠️ Your project config has an error, check your `wrangler.toml`: {}",
                e
            );
            Err(failure::err_msg(msg))
        }
    }
}
