use std::env;
use std::time::Duration;

use cloudflare::framework::async_api;
use cloudflare::framework::auth::Credentials;
use cloudflare::framework::response::ApiFailure;
use cloudflare::framework::{Environment, HttpApiClient, HttpApiClientConfig};
use http::StatusCode;

use anyhow::Result;

use crate::http::{feature::headers, DEFAULT_HTTP_TIMEOUT_SECONDS};
use crate::settings::global_user::GlobalUser;
use crate::terminal::emoji;
use crate::terminal::message::{Message, StdOut};

const CF_API_BASE_URL: &str = "CF_API_BASE_URL";

// Allow endpoint to be configured via an environment variable
pub fn get_environment() -> Result<Environment> {
    let env_hostname = match env::var(CF_API_BASE_URL) {
        Ok(value) => {
            if let Ok(url) = url::Url::parse(&value) {
                url
            } else {
                anyhow::bail!("Failed to parse URL from environment variable. Please make sure your API endpoint URL is valid.")
            }
        }
        Err(_) => return Ok(Environment::Production),
    };

    Ok(Environment::Custom(env_hostname))
}

pub fn cf_v4_client(user: &GlobalUser) -> Result<HttpApiClient> {
    let config = HttpApiClientConfig {
        http_timeout: Duration::from_secs(DEFAULT_HTTP_TIMEOUT_SECONDS),
        default_headers: headers(None),
    };

    let environment = get_environment()?;

    HttpApiClient::new(Credentials::from(user.to_owned()), config, environment)
}

pub fn cf_v4_api_client_async(user: &GlobalUser) -> Result<async_api::Client> {
    let config = HttpApiClientConfig {
        http_timeout: Duration::from_secs(DEFAULT_HTTP_TIMEOUT_SECONDS),
        default_headers: headers(None),
    };

    let environment = get_environment()?;

    async_api::Client::new(Credentials::from(user.to_owned()), config, environment)
}

// Format errors from the cloudflare-rs cli for printing.
// Optionally takes an argument for providing a function that maps error code numbers to
// helpful additional information about why someone is getting an error message and how to fix it.
pub fn format_error(e: ApiFailure, err_helper: Option<&dyn Fn(u16) -> &'static str>) -> String {
    match e {
        ApiFailure::Error(status, api_errors) => {
            print_status_code_context(status);
            let mut complete_err = "".to_string();
            for error in api_errors.errors {
                let error_msg = format!("{} Code {}: {}\n", emoji::WARN, error.code, error.message);

                if let Some(annotate_help) = err_helper {
                    let suggestion_text = annotate_help(error.code);
                    let help_msg = format!("{} {}\n", emoji::SLEUTH, suggestion_text);
                    complete_err.push_str(&format!("{}{}", error_msg, help_msg));
                } else {
                    complete_err.push_str(&error_msg)
                }
            }
            complete_err.trim_end().to_string() // Trimming strings in place for String is apparently not a thing...
        }
        ApiFailure::Invalid(reqwest_err) => format!("{} Error: {}", emoji::WARN, reqwest_err),
    }
}

// For handling cases where the API gateway returns errors via HTTP status codes
// (no API-specific, more granular error code is given).
fn print_status_code_context(status_code: StatusCode) {
    match status_code {
      // Folks should never hit PAYLOAD_TOO_LARGE, given that Wrangler ensures that bulk file uploads
      // are max ~50 MB in size. This case is handled anyways out of an abundance of caution.
      StatusCode::PAYLOAD_TOO_LARGE => StdOut::warn("Returned status code 413, Payload Too Large. Please make sure your upload is less than 100MB in size"),
      StatusCode::GATEWAY_TIMEOUT => StdOut::warn("Returned status code 504, Gateway Timeout. Please try again in a few seconds"),
      _ => (),
  }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn gets_environment_tests() {
        // Tests are run in parallel, so dividing these three tests can lead
        // to an undesired interleaving in set and unset of the environment variables.

        // Test #1
        // Tests that the API endpoint base URL can be read from an environment variable
        let url = "https://github.com/cloudflare/wrangler";

        env::set_var(CF_API_BASE_URL, url);
        let test_environment_url = url::Url::from(&get_environment().unwrap());

        let expected_environment_url = url::Url::parse(url).unwrap();
        assert_eq!(test_environment_url, expected_environment_url);
        env::remove_var(CF_API_BASE_URL);

        // Test #2
        // Tests that it fails with an invalid url
        let url = "thisisaninvalidurl";

        env::set_var(CF_API_BASE_URL, url);
        let test_environment = get_environment();

        assert!(test_environment.is_err());
        env::remove_var(CF_API_BASE_URL);

        // Test #3
        // Tests that the API endpoint base URL is set to production if no environment variable is set

        // unset the environment variable
        env::remove_var(CF_API_BASE_URL);
        let test_environment_url = url::Url::from(&get_environment().unwrap());

        let expected_environment_url = url::Url::from(&Environment::Production);
        assert_eq!(test_environment_url, expected_environment_url);
    }
}
