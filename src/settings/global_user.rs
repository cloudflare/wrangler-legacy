use std::fs;
use std::path::{Path, PathBuf};

use anyhow::Result;
use cloudflare::framework::auth::Credentials;
use serde::{Deserialize, Serialize};

use crate::login::check_update_oauth_token;
use crate::settings::{get_global_config_path, Environment, QueryEnvironment};
use crate::terminal::{emoji, styles};

const CF_API_TOKEN: &str = "CF_API_TOKEN";
const CF_API_KEY: &str = "CF_API_KEY";
const CF_EMAIL: &str = "CF_EMAIL";

static ENV_VAR_WHITELIST: [&str; 3] = [CF_API_TOKEN, CF_API_KEY, CF_EMAIL];

#[cfg(test)]
use std::io::Write;

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
#[serde(untagged)]
pub enum GlobalUser {
    ApiTokenAuth {
        api_token: String,
    },
    OAuthTokenAuth {
        oauth_token: String,
        refresh_token: String,
        expiration_time: String,
    },
    GlobalKeyAuth {
        email: String,
        api_key: String,
    },
}

impl GlobalUser {
    pub fn new() -> Result<Self> {
        let environment = Environment::with_whitelist(ENV_VAR_WHITELIST.to_vec());

        let config_path = get_global_config_path();
        let mut new_user = GlobalUser::build(environment, config_path);

        // Check if oauth token is expired
        if let Ok(ref mut oauth_user) = new_user {
            // Let caller handle possible errors
            check_update_oauth_token(oauth_user)?;
        }
        new_user
    }

    fn build<T: 'static + QueryEnvironment>(environment: T, config_path: PathBuf) -> Result<Self>
    where
        T: config::Source + Send + Sync,
    {
        if let Some(user) = Self::from_env(environment) {
            user
        } else {
            Self::from_file(config_path)
        }
    }

    pub fn get_expiration_time(&self) -> &String {
        match self {
            GlobalUser::OAuthTokenAuth {
                oauth_token: _,
                refresh_token: _,
                expiration_time,
            } => expiration_time,
            _ => unreachable!(),
        }
    }

    pub fn get_refresh_token(&self) -> &String {
        match self {
            GlobalUser::OAuthTokenAuth {
                oauth_token: _,
                refresh_token,
                expiration_time: _,
            } => refresh_token,
            _ => unreachable!(),
        }
    }

    pub fn set_oauth_token(&mut self, new_oauth_token: String) {
        match self {
            GlobalUser::OAuthTokenAuth {
                ref mut oauth_token,
                refresh_token: _,
                expiration_time: _,
            } => *oauth_token = new_oauth_token,
            _ => unreachable!(),
        }
    }

    pub fn set_refresh_token(&mut self, new_refresh_token: String) {
        match self {
            GlobalUser::OAuthTokenAuth {
                oauth_token: _,
                ref mut refresh_token,
                expiration_time: _,
            } => *refresh_token = new_refresh_token,
            _ => unreachable!(),
        }
    }

    pub fn set_expiration_time(&mut self, new_expiration_time: String) {
        match self {
            GlobalUser::OAuthTokenAuth {
                oauth_token: _,
                refresh_token: _,
                expiration_time,
            } => *expiration_time = new_expiration_time,
            _ => unreachable!(),
        }
    }

    fn from_env<T: 'static + QueryEnvironment>(environment: T) -> Option<Result<Self>>
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

    fn from_file(config_path: PathBuf) -> Result<Self> {
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

            match s.merge(config::File::with_name(config_str)) {
                Ok(_) => (),
                Err(_) => {
                    let error_info = "\nFailed to read information from configuration file.";
                    return Self::show_config_err_info(Some(error_info.to_string()), s);
                }
            }
        } else {
            anyhow::bail!(
                "config path does not exist {}. Try running `wrangler login` or `wrangler config`",
                config_str
            );
        }

        GlobalUser::from_config(s)
    }

    pub fn to_file(&self, config_path: &Path) -> Result<()> {
        // convert in-memory representation of authentication method to on-disk format
        let toml: std::string::String = toml::to_string(self)?;

        // create configuration path if non-existent, and write to configuration file
        fs::create_dir_all(&config_path.parent().unwrap())?;
        fs::write(&config_path, toml)?;

        Ok(())
    }

    fn from_config(config: config::Config) -> Result<Self> {
        // Get all the possible authentication methods
        let api_token = config.get_str("api_token");
        let oauth_token = config.get_str("oauth_token");
        let email = config.get_str("email");
        let api_key = config.get_str("api_key");
        let refresh_token = config.get_str("refresh_token");
        let expiration_time = config.get_str("expiration_time");

        // The only cases that are not allowed are:
        //      1) (partial or complete) OAuth token + API token
        //      2) (partial or complete) OAuth token + Global API key (partial or complete)
        //      3) Invalid authentication methods (e.g. partial Oauth token, partial Global API key, and empty configuration file + no environment variables)
        // API token has priority over global API key both in environment variables and in configuration file
        if (api_token.is_ok()
            && (oauth_token.is_ok() || refresh_token.is_ok() || expiration_time.is_ok()))
            || ((oauth_token.is_ok() || refresh_token.is_ok() || expiration_time.is_ok())
                && (email.is_ok() || api_key.is_ok()))
        {
            let error_info = "\nMore than one authentication method (e.g. API token and OAuth token, or OAuth token and Global API key) has been found in the configuration file. Please use only one.";
            let wrangler_logout_msg = styles::highlight("`wrangler logout`");
            let config_path = get_global_config_path();
            let more_info = format!("{}\nIf you'd like to edit the configuration file, it can be found at {}. Consider also running {} to clean up the configuration file.", error_info, config_path.to_str().unwrap(), wrangler_logout_msg);

            Self::show_config_err_info(Some(more_info), config)
        } else if let Ok(api_token) = api_token {
            Ok(Self::ApiTokenAuth { api_token })
        } else if let (Ok(email), Ok(api_key)) = (email, api_key) {
            Ok(Self::GlobalKeyAuth { email, api_key })
        } else if let (Ok(oauth_token), Ok(refresh_token), Ok(expiration_time)) =
            (oauth_token, refresh_token, expiration_time)
        {
            Ok(Self::OAuthTokenAuth {
                oauth_token,
                refresh_token,
                expiration_time,
            })
        } else {
            // Empty configuration file and no environment variables, or missing variable for global API key
            let error_info = "\nNo (valid) authentication method has been found.";
            Self::show_config_err_info(Some(error_info.to_string()), config)
        }
    }

    fn show_config_err_info(
        info: Option<std::string::String>,
        config: config::Config,
    ) -> Result<Self> {
        let wrangler_login_msg = styles::highlight("`wrangler login`");
        let wrangler_config_msg = styles::highlight("`wrangler config`");
        let vars_msg = styles::url("https://developers.cloudflare.com/workers/tooling/wrangler/configuration/#using-environment-variables");
        let additional_info = match info {
            Some(text) => text,
            None => "".to_string(),
        };

        let msg = format!(
            "{} Your authentication details are improperly configured.{}\nPlease run {}, {}, or visit\n{}\nfor info on configuring with environment variables",
            emoji::WARN,
            additional_info,
            wrangler_login_msg,
            wrangler_config_msg,
            vars_msg
        );

        log::info!("{:?}", config);
        anyhow::bail!(msg)
    }
}

impl From<GlobalUser> for Credentials {
    fn from(user: GlobalUser) -> Credentials {
        match user {
            GlobalUser::ApiTokenAuth { api_token } => {
                Credentials::UserAuthToken { token: api_token }
            }
            GlobalUser::OAuthTokenAuth {
                oauth_token,
                refresh_token: _,
                expiration_time: _,
            } => Credentials::UserAuthToken { token: oauth_token },
            GlobalUser::GlobalKeyAuth { email, api_key } => Credentials::UserAuthKey {
                key: api_key,
                email,
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::File;
    use tempfile::tempdir;

    use crate::settings::{environment::MockEnvironment, DEFAULT_CONFIG_FILE_NAME};

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
            GlobalUser::ApiTokenAuth {
                api_token: "foo".to_string(),
            }
        );
    }

    #[test]
    fn it_can_prioritize_env_vars() {
        let api_token = "thisisanapitoken";
        let api_key = "reallylongglobalapikey";
        let email = "user@example.com";

        let file_user = GlobalUser::ApiTokenAuth {
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

        let user = GlobalUser::ApiTokenAuth {
            api_token: "thisisanapitoken".to_string(),
        };

        let tmp_dir = tempdir().unwrap();
        let tmp_config_path = test_config_dir(&tmp_dir, Some(user.clone())).unwrap();

        let new_user = GlobalUser::build(mock_env, tmp_config_path).unwrap();

        assert_eq!(new_user, user);
    }

    #[test]
    fn it_succeeds_with_oauth_token() {
        let user = GlobalUser::OAuthTokenAuth {
            oauth_token: "thisisanoauthtoken".to_string(),
            refresh_token: "thisisarefreshtoken".to_string(),
            expiration_time: "thisisexpirationtime".to_string(),
        };

        let tmp_dir = tempdir().unwrap();
        let tmp_config_path = test_config_dir(&tmp_dir, Some(user.clone())).unwrap();

        let file_user = GlobalUser::from_file(tmp_config_path).unwrap();
        assert_eq!(file_user, user);
    }

    #[test]
    fn it_fails_if_oauth_token_incomplete_in_file() {
        let tmp_dir = tempdir().unwrap();
        let config_dir = test_config_dir(&tmp_dir, None).unwrap();

        let mut file = fs::OpenOptions::new()
            .write(true)
            .open(&config_dir.as_path())
            .unwrap();

        let oauth_token = "oauth_token = \"thisisanoauthtoken\"";
        file.write_all(oauth_token.as_bytes()).unwrap();

        let file_user = GlobalUser::from_file(config_dir);

        assert!(file_user.is_err());
    }

    #[test]
    fn it_fails_if_oauth_and_api_tokens_both_exist() {
        // This test checks whether GlobalUser returns an error
        // when both api_token and oauth_token are set in the config file.
        // Expected behavior: the newly created GlobalUser should return an error.
        let user = GlobalUser::ApiTokenAuth {
            api_token: "thisisanapitoken".to_string(),
        };

        let user_extra_toml: std::string::String = toml::to_string(&GlobalUser::OAuthTokenAuth {
            oauth_token: "thisisanoauthtoken".to_string(),
            refresh_token: "thisisarefreshtoken".to_string(),
            expiration_time: "thisisexpirationtime".to_string(),
        })
        .unwrap();

        let tmp_dir = tempdir().unwrap();
        let tmp_config_path = test_config_dir(&tmp_dir, Some(user.clone())).unwrap();

        let mut temp_file = fs::OpenOptions::new()
            .append(true)
            .open(&tmp_config_path)
            .unwrap();
        temp_file.write(user_extra_toml.as_bytes()).unwrap();

        let file_user = GlobalUser::from_file(tmp_config_path);
        assert!(file_user.is_err());
    }

    #[test]
    fn it_fails_if_partial_oauth_and_api_tokens_both_exist() {
        // This test checks whether GlobalUser returns an error
        // when both api_token and refresh_token are set in the config file.
        // Expected behavior: the newly created GlobalUser should return an error.
        let user = GlobalUser::ApiTokenAuth {
            api_token: "thisisanapitoken".to_string(),
        };

        let user_extra_toml = "refresh_token: \"thisisarefreshtoken\"";

        let tmp_dir = tempdir().unwrap();
        let tmp_config_path = test_config_dir(&tmp_dir, Some(user.clone())).unwrap();

        let mut temp_file = fs::OpenOptions::new()
            .append(true)
            .open(&tmp_config_path)
            .unwrap();
        temp_file.write(user_extra_toml.as_bytes()).unwrap();

        let file_user = GlobalUser::from_file(tmp_config_path);
        assert!(file_user.is_err());
    }

    #[test]
    fn it_fails_if_oauth_token_and_complete_global_key_both_exist() {
        // This test checks whether GlobalUser returns an error
        // when both global api key with email and oauth_token are set in the config file.
        // Expected behavior: the newly created GlobalUser should return an error.
        let user = GlobalUser::GlobalKeyAuth {
            email: "user@example.com".to_string(),
            api_key: "reallylongglobalapikey".to_string(),
        };

        let user_extra_toml: std::string::String = toml::to_string(&GlobalUser::OAuthTokenAuth {
            oauth_token: "thisisanoauthtoken".to_string(),
            refresh_token: "thisisarefreshtoken".to_string(),
            expiration_time: "thisisexpirationtime".to_string(),
        })
        .unwrap();

        let tmp_dir = tempdir().unwrap();
        let tmp_config_path = test_config_dir(&tmp_dir, Some(user.clone())).unwrap();

        let mut temp_file = fs::OpenOptions::new()
            .append(true)
            .open(&tmp_config_path)
            .unwrap();
        temp_file.write(user_extra_toml.as_bytes()).unwrap();

        let file_user = GlobalUser::from_file(tmp_config_path);
        assert!(file_user.is_err());
    }

    #[test]
    fn it_fails_if_partial_oauth_token_and_complete_global_key_both_exist() {
        // This test checks whether GlobalUser returns an error
        // when both global api key with email and refresh_token are set in the config file.
        // Expected behavior: the newly created GlobalUser should return an error.
        let user = GlobalUser::GlobalKeyAuth {
            email: "user@example.com".to_string(),
            api_key: "reallylongglobalapikey".to_string(),
        };

        let user_extra_toml = "refresh_token: \"thisisarefreshtoken\"";

        let tmp_dir = tempdir().unwrap();
        let tmp_config_path = test_config_dir(&tmp_dir, Some(user.clone())).unwrap();

        let mut temp_file = fs::OpenOptions::new()
            .append(true)
            .open(&tmp_config_path)
            .unwrap();
        temp_file.write(user_extra_toml.as_bytes()).unwrap();

        let file_user = GlobalUser::from_file(tmp_config_path);
        assert!(file_user.is_err());
    }

    #[test]
    fn it_fails_if_oauth_token_and_user_email_both_exist() {
        // This test checks whether GlobalUser returns an error
        // when both email without globaly key and oauth_token are set in the config file.
        // Expected behavior: the newly created GlobalUser should return an error.
        let email_config = "email = \"user@example.com\"";

        let user_extra_toml: std::string::String = toml::to_string(&GlobalUser::OAuthTokenAuth {
            oauth_token: "thisisanoauthtoken".to_string(),
            refresh_token: "thisisarefreshtoken".to_string(),
            expiration_time: "thisisexpirationtime".to_string(),
        })
        .unwrap();

        let tmp_dir = tempdir().unwrap();
        let tmp_config_path = test_config_dir(&tmp_dir, None).unwrap();

        let mut temp_file = fs::OpenOptions::new()
            .append(true)
            .open(&tmp_config_path)
            .unwrap();

        temp_file.write(email_config.as_bytes()).unwrap();
        temp_file.write(user_extra_toml.as_bytes()).unwrap();

        let file_user = GlobalUser::from_file(tmp_config_path);
        assert!(file_user.is_err());
    }

    #[test]
    fn it_fails_if_oauth_token_and_global_key_without_user_email_both_exist() {
        // This test checks whether GlobalUser returns an error
        // when both global api key without email and oauth_token are set in the config file.
        // Expected behavior: the newly created GlobalUser should return an error.
        let api_key = "api_key = \"reallylongglobalapikey\"";

        let user_extra_toml: std::string::String = toml::to_string(&GlobalUser::OAuthTokenAuth {
            oauth_token: "thisisanoauthtoken".to_string(),
            refresh_token: "thisisarefreshtoken".to_string(),
            expiration_time: "thisisexpirationtime".to_string(),
        })
        .unwrap();

        let tmp_dir = tempdir().unwrap();
        let tmp_config_path = test_config_dir(&tmp_dir, None).unwrap();

        let mut temp_file = fs::OpenOptions::new()
            .append(true)
            .open(&tmp_config_path)
            .unwrap();

        temp_file.write(api_key.as_bytes()).unwrap();
        temp_file.write(user_extra_toml.as_bytes()).unwrap();

        let file_user = GlobalUser::from_file(tmp_config_path);
        assert!(file_user.is_err());
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

    #[test]
    fn it_succeeds_with_no_config() {
        let mut mock_env = MockEnvironment::default();
        mock_env.set(CF_API_KEY, "apikey");
        mock_env.set(CF_EMAIL, "email");
        let dummy_path = Path::new("./definitely-does-not-exist.txt").to_path_buf();
        let new_user = GlobalUser::build(mock_env, dummy_path);

        assert!(new_user.is_ok());
    }

    #[test]
    fn it_fails_with_no_authentication_method() {
        let mock_env = MockEnvironment::default();
        let tmp_dir = tempdir().unwrap();
        let config_dir = test_config_dir(&tmp_dir, None).unwrap();

        let new_user = GlobalUser::build(mock_env, config_dir);

        assert!(new_user.is_err());
    }

    fn test_config_dir(tmp_dir: &tempfile::TempDir, user: Option<GlobalUser>) -> Result<PathBuf> {
        let tmp_config_path = tmp_dir.path().join(DEFAULT_CONFIG_FILE_NAME);
        if let Some(user_config) = user {
            user_config.to_file(&tmp_config_path)?;
        } else {
            File::create(&tmp_config_path)?;
        }

        Ok(tmp_config_path)
    }
}
