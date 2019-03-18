use config::{Config, Environment, File};
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Clone, Serialize)]
pub struct Settings {
    pub email: String,
    pub api_key: String,
}

impl Settings {
    pub fn new() -> Result<Self, failure::Error> {
        let mut s = Config::new();

        let config_path = dirs::home_dir()
            .expect("oops no home dir")
            .join(".wrangler/config/default");
        let config_str = config_path
            .to_str()
            .expect("config path should be a string");
        s.merge(File::with_name(config_str))?;

        // Eg.. `CF_ACCOUNT_AUTH_KEY=farts` would set the `account_auth_key` key
        s.merge(Environment::with_prefix("CF"))?;

        Ok(s.try_into()?)
    }
}
