use reqwest::header::{HeaderMap, HeaderValue, USER_AGENT};
use reqwest::{Client, ClientBuilder, RedirectPolicy};
use std::time::Duration;

use crate::install;
use crate::settings::global_user::GlobalUser;

use http::status::StatusCode;

use cloudflare::framework::auth::Credentials;
use cloudflare::framework::response::ApiFailure;
use cloudflare::framework::{Environment, HttpApiClient, HttpApiClientConfig};

use crate::terminal::emoji;
use crate::terminal::message;

////---------------------------OLD API CLIENT CODE---------------------------////
// TODO: remove this and replace it entirely with cloudflare-rs
fn headers(feature: Option<&str>) -> HeaderMap {
    let version = if install::target::DEBUG {
        "dev"
    } else {
        env!("CARGO_PKG_VERSION")
    };
    let user_agent = if let Some(feature) = feature {
        format!("wrangler/{}/{}", version, feature)
    } else {
        format!("wrangler/{}", version)
    };

    let mut headers = HeaderMap::new();
    headers.insert(USER_AGENT, HeaderValue::from_str(&user_agent).unwrap());
    headers
}

fn builder() -> ClientBuilder {
    let builder = reqwest::Client::builder();
    builder
        .connect_timeout(Duration::from_secs(10))
        .timeout(Duration::from_secs(30))
}

pub fn client(feature: Option<&str>) -> Client {
    builder()
        .default_headers(headers(feature))
        .build()
        .expect("could not create http client")
}

pub fn auth_client(feature: Option<&str>, user: &GlobalUser) -> Client {
    let mut headers = headers(feature);
    add_auth_headers(&mut headers, user);

    builder()
        .default_headers(headers.to_owned())
        .redirect(RedirectPolicy::none())
        .build()
        .expect("could not create authenticated http client")
}

fn add_auth_headers<'a>(headers: &'a mut HeaderMap, user: &GlobalUser) {
    match user {
        GlobalUser::TokenAuth { api_token } => {
            headers.insert(
                "Authorization",
                HeaderValue::from_str(&format!("Bearer {}", &api_token)).unwrap(),
            );
        }
        GlobalUser::GlobalKeyAuth { email, api_key } => {
            headers.insert("X-Auth-Email", HeaderValue::from_str(&email).unwrap());
            headers.insert("X-Auth-Key", HeaderValue::from_str(&api_key).unwrap());
        }
    }
}

////---------------------------NEW API CLIENT CODE---------------------------////
pub fn api_client(
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
            complete_err
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
        StatusCode::PAYLOAD_TOO_LARGE =>
            message::warn("Returned status code 413, Payload Too Large. Please make sure your upload is less than 100MB in size"),
        StatusCode::GATEWAY_TIMEOUT =>
            message::warn("Returned status code 504, Gateway Timeout. Please try again in a few seconds"),
        _ => (),
    }
}
