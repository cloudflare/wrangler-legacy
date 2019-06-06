use config::{Config, Environment, File};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct GlobalUser {
    pub email: String,
    pub api_key: String,
}

impl GlobalUser {
    pub fn new() -> Result<Self, failure::Error> {
        get_global_config()
    }
    pub fn config_directory() -> Option<PathBuf> {
        get_global_config_directory()
    }
}

fn get_global_config_directory() -> Option<PathBuf> {
    let legacy_directory = dirs::home_dir()
        .expect("oops no home dir")
        .join(".wrangler")
        .join("config");

    let directory;
    if legacy_directory.exists() {
        directory = Some(legacy_directory);
    } else {
        directory = dirs::config_dir().map(|p| p.join("wrangler"));
    }

    directory
}

fn get_global_config() -> Result<GlobalUser, failure::Error> {
    let mut s = Config::new();

    let config_str = get_global_config_directory()
        .expect("oops no config dir")
        .join("default")
        .to_str()
        .expect("global config path should be a string")
        .to_owned();
    s.merge(File::with_name(&config_str))?;

    // Eg.. `CF_ACCOUNT_AUTH_KEY=farts` would set the `account_auth_key` key
    s.merge(Environment::with_prefix("CF"))?;

    let global_user: Result<GlobalUser, config::ConfigError> = s.try_into();
    match global_user {
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
