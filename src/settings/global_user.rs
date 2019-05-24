use config::{Config, Environment, File};
use serde::{Deserialize, Serialize};

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

    let config_path = dirs::home_dir()
        .expect("oops no home dir")
        .join(".wrangler/config/default");
    let config_str = config_path
        .to_str()
        .expect("global config path should be a string");
    s.merge(File::with_name(config_str))?;

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
