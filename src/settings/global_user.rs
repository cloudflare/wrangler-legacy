use std::env;
use std::fs;
use std::io::prelude::*;
use std::path::{Path, PathBuf};

use cloudflare::framework::auth::Credentials;
use config;
use serde::{Deserialize, Serialize};

use crate::settings::{Environment, QueryEnvironment};
use crate::terminal::emoji;

const DEFAULT_CONFIG_FILE_NAME: &str = "default.toml";

const CF_API_TOKEN: &str = "CF_API_TOKEN";
const CF_API_KEY: &str = "CF_API_KEY";
const CF_EMAIL: &str = "CF_EMAIL";

static ENV_VAR_WHITELIST: [&str; 3] = [CF_API_TOKEN, CF_API_KEY, CF_EMAIL];

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
#[serde(untagged)]
pub enum GlobalUser {
    TokenAuth { api_token: String },
    GlobalKeyAuth { email: String, api_key: String },
}

impl GlobalUser {
    pub fn new() -> Result<Self, failure::Error> {
        let environment = Environment::with_whitelist(ENV_VAR_WHITELIST.to_vec());

        let config_path = default_config_file().expect("could not find global config directory");

        GlobalUser::build(environment, config_path)
    }

    fn build<T: 'static + QueryEnvironment>(
        environment: T,
        config_path: PathBuf,
    ) -> Result<Self, failure::Error>
    where
        T: config::Source + Send + Sync,
    {
        if let Some(user) = Self::from_env(environment) {
            user
        } else {
            Self::from_file(config_path)
        }
    }

    fn from_env<T: 'static + QueryEnvironment>(
        environment: T,
    ) -> Option<Result<Self, failure::Error>>
    where
        T: config::Source + Send + Sync,
    {
        // if there's some problem with gathering the environment,
        // or if there are no relevant environment variables set,
        // fall back to config file.
        if environment.empty().unwrap_or(true) {
            None
        } else {
            let mut s = config::Config::new();
            s.merge(environment).ok();

            Some(GlobalUser::from_config(s))
        }
    }

    fn from_file(config_path: PathBuf) -> Result<Self, failure::Error> {
        let mut s = config::Config::new();

        let config_str = config_path
            .to_str()
            .expect("global config path should be a string");

        // Skip reading global config if non existent
        // because envs might be provided
        if config_path.exists() {
            log::info!(
                "Config path exists. Reading from config file, {}",
                config_str
            );
            s.merge(config::File::with_name(config_str))?;
        } else {
            failure::bail!(
                "config path does not exist {}. Try running `wrangler config`",
                config_str
            );
        }

        GlobalUser::from_config(s)
    }

    pub fn to_file(&self, config_path: &Path) -> Result<(), failure::Error> {
        let toml = toml::to_string(self)?;

        fs::create_dir_all(&config_path.parent().unwrap())?;
        fs::write(&config_path, toml)?;

        Ok(())
    }

    fn from_config(config: config::Config) -> Result<Self, failure::Error> {
        let global_user: Result<GlobalUser, config::ConfigError> = config.clone().try_into();
        match global_user {
            Ok(user) => Ok(user),
            Err(_) => {
                let msg = format!(
                    "{} Your authentication details are improperly configured, please run `wrangler config`",
                    emoji::WARN,
                );
                log::info!("{:?}", config);
                failure::bail!(msg)
            }
        }
    }
}

impl From<GlobalUser> for Credentials {
    fn from(user: GlobalUser) -> Credentials {
        match user {
            GlobalUser::TokenAuth { api_token } => Credentials::UserAuthToken { token: api_token },
            GlobalUser::GlobalKeyAuth { email, api_key } => Credentials::UserAuthKey {
                key: api_key,
                email,
            },
        }
    }
}

pub fn default_config_file() -> Result<PathBuf, failure::Error> {
    let home_dir = if let Ok(value) = env::var("WRANGLER_HOME") {
        log::info!("Using WRANGLER_HOME: {}", value);
        Path::new(&value).to_path_buf()
    } else {
        log::info!("No WRANGLER_HOME detected");
        dirs::home_dir()
            .expect("oops no home dir")
            .join(".wrangler")
    };
    let global_config_file = home_dir.join("config").join(DEFAULT_CONFIG_FILE_NAME);
    log::info!("Using global config file: {}", global_config_file.display());
    Ok(global_config_file)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::File;
    use tempfile::tempdir;

    use crate::settings::environment::MockEnvironment;

    #[test]
    fn it_can_prioritize_token_input() {
        // Set all CF_API_TOKEN, CF_EMAIL, and CF_API_KEY.
        // This test evaluates whether the GlobalUser returned is
        // a GlobalUser::TokenAuth (expected behavior; token
        // should be prioritized over email + global API key pair.)
        let mut mock_env = MockEnvironment::default();
        mock_env.set(CF_API_TOKEN, "foo");
        mock_env.set(CF_EMAIL, "test@cloudflare.com");
        mock_env.set(CF_API_KEY, "bar");

        let tmp_dir = tempdir().unwrap();
        let config_dir = test_config_dir(&tmp_dir, None).unwrap();

        let user = GlobalUser::build(mock_env, config_dir).unwrap();
        assert_eq!(
            user,
            GlobalUser::TokenAuth {
                api_token: "foo".to_string()
            }
        );
    }

    #[test]
    fn it_can_prioritize_env_vars() {
        let api_token = "thisisanapitoken";
        let api_key = "reallylongglobalapikey";
        let email = "user@example.com";

        let file_user = GlobalUser::TokenAuth {
            api_token: api_token.to_string(),
        };
        let env_user = GlobalUser::GlobalKeyAuth {
            api_key: api_key.to_string(),
            email: email.to_string(),
        };

        let mut mock_env = MockEnvironment::default();
        mock_env.set(CF_EMAIL, email);
        mock_env.set(CF_API_KEY, api_key);

        let tmp_dir = tempdir().unwrap();
        let tmp_config_path = test_config_dir(&tmp_dir, Some(file_user)).unwrap();

        let new_user = GlobalUser::build(mock_env, tmp_config_path).unwrap();

        assert_eq!(new_user, env_user);
    }

    #[test]
    fn it_falls_through_to_config_with_no_env_vars() {
        let mock_env = MockEnvironment::default();

        let user = GlobalUser::TokenAuth {
            api_token: "thisisanapitoken".to_string(),
        };

        let tmp_dir = tempdir().unwrap();
        let tmp_config_path = test_config_dir(&tmp_dir, Some(user.clone())).unwrap();

        let new_user = GlobalUser::build(mock_env, tmp_config_path).unwrap();

        assert_eq!(new_user, user);
    }

    #[test]
    fn it_fails_if_global_auth_incomplete_in_file() {
        let tmp_dir = tempdir().unwrap();
        let config_dir = test_config_dir(&tmp_dir, None).unwrap();

        let mut file = fs::OpenOptions::new()
            .write(true)
            .open(&config_dir.as_path())
            .unwrap();
        let email_config = "email = \"thisisanemail\"";
        file.write_all(email_config.as_bytes()).unwrap();

        let file_user = GlobalUser::from_file(config_dir);

        assert!(file_user.is_err());
    }

    #[test]
    fn it_fails_if_global_auth_incomplete_in_env() {
        let mut mock_env = MockEnvironment::default();

        mock_env.set(CF_API_KEY, "apikey");

        let tmp_dir = tempdir().unwrap();
        let config_dir = test_config_dir(&tmp_dir, None).unwrap();

        let new_user = GlobalUser::build(mock_env, config_dir);

        assert!(new_user.is_err());
    }

    fn test_config_dir(
        tmp_dir: &tempfile::TempDir,
        user: Option<GlobalUser>,
    ) -> Result<PathBuf, failure::Error> {
        let tmp_config_path = tmp_dir.path().join(DEFAULT_CONFIG_FILE_NAME);
        if let Some(user_config) = user {
            user_config.to_file(&tmp_config_path)?;
        } else {
            File::create(&tmp_config_path)?;
        }

        Ok(tmp_config_path.to_path_buf())
    }
}
