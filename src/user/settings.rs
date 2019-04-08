use std::collections::HashMap;
use std::path::Path;

use config::{Config, Environment, File};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct GlobalSettings {
    pub email: String,
    pub api_key: String,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ProjectSettings {
    pub zone_id: String,
    pub account_id: String,
    pub route: Option<String>,
    pub routes: Option<HashMap<String, String>>,
}

#[derive(Clone, Serialize)]
pub struct Settings {
    pub global: GlobalSettings,
    pub project: ProjectSettings,
}

impl Settings {
    pub fn new() -> Result<Self, failure::Error> {
        let global = get_global_config()?;
        let project = get_project_config()?;

        Ok(Settings { global, project })
    }
}

fn get_global_config() -> Result<GlobalSettings, failure::Error> {
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

    Ok(s.try_into()?)
}

fn get_project_config() -> Result<ProjectSettings, failure::Error> {
    let mut s = Config::new();

    let config_path = Path::new("./wrangler.toml");
    let config_str = config_path
        .to_str()
        .expect("project config path should be a string");
    s.merge(File::with_name(config_str))?;

    // Eg.. `CF_ACCOUNT_AUTH_KEY=farts` would set the `account_auth_key` key
    s.merge(Environment::with_prefix("CF"))?;

    Ok(s.try_into()?)
}
