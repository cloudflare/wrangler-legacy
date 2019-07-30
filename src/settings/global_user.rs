use std::env;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::terminal::emoji;
use config::{Config, Environment, File};

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct GlobalUser {
    pub email: String,
    pub api_key: String,
}

impl GlobalUser {
    pub fn new() -> Result<Self, failure::Error> {
        get_global_config()
    }
}

fn get_global_config() -> Result<GlobalUser, failure::Error> {
    let mut s = Config::new();

    let config_path = get_global_config_dir()
        .expect("could not find global config directory")
        .join("default");
    let config_str = config_path
        .to_str()
        .expect("global config path should be a string");

    // Skip reading global config if non existent
    // because envs might be provided
    if config_path.exists() {
        s.merge(File::with_name(config_str))?;
    }

    // Eg.. `CF_API_KEY=farts` would set the `account_auth_key` key
    // envs are: CF_API_KEY and CF_EMAIL
    s.merge(Environment::with_prefix("CF"))?;

    let global_user: Result<GlobalUser, config::ConfigError> = s.try_into();
    match global_user {
        Ok(s) => Ok(s),
        Err(e) => {
            let msg = format!(
                "{} Your global config has an error, run `wrangler config`: {}",
                emoji::WARN,
                e
            );
            failure::bail!(msg)
        }
    }
}

pub fn get_global_config_dir() -> Result<PathBuf, failure::Error> {
    let home_dir = if let Ok(value) = env::var("WRANGLER_HOME") {
        Path::new(&value).to_path_buf()
    } else {
        dirs::home_dir()
            .expect("oops no home dir")
            .join(".wrangler")
    };

    Ok(home_dir.join("config"))
}
