use std::time::Duration;

use cloudflare::framework::auth::Credentials;
use cloudflare::framework::response::ApiFailure;
use cloudflare::framework::{Environment, HttpApiClient, HttpApiClientConfig};
use http::StatusCode;

use crate::http::{feature::headers, Feature};
use crate::settings::global_user::GlobalUser;
use crate::terminal::{emoji, message};

pub struct CfApiClientConfig {
    feature: Option<Feature>,
    http_timeout: Duration,
}

impl CfApiClientConfig {
    pub fn default() -> HttpApiClientConfig {
        let mut config = Self::builder();
        config.build()
    }

    pub fn builder() -> Self {
        CfApiClientConfig {
            http_timeout: Duration::from_secs(30),
            feature: None,
        }
    }

    pub fn timeout<'a>(&'a mut self, timeout: Duration) -> &'a mut CfApiClientConfig {
        self.http_timeout = timeout;
        self
    }

    pub fn feature<'a>(&'a mut self, feature: Feature) -> &'a mut CfApiClientConfig {
        self.feature = Some(feature);
        self
    }

    pub fn build<'a>(&'a mut self) -> HttpApiClientConfig {
        HttpApiClientConfig {
            http_timeout: self.http_timeout,
            default_headers: headers(self.feature),
        }
    }
}

pub fn cf_api_client(
    user: &GlobalUser,
    config: HttpApiClientConfig,
) -> Result<HttpApiClient, failure::Error> {
    HttpApiClient::new(
        Credentials::from(user.to_owned()),
        config,
        Environment::Production,
    )
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
      StatusCode::PAYLOAD_TOO_LARGE => message::warn("Returned status code 413, Payload Too Large. Please make sure your upload is less than 100MB in size"),
      StatusCode::GATEWAY_TIMEOUT => message::warn("Returned status code 504, Gateway Timeout. Please try again in a few seconds"),
      _ => (),
  }
}
