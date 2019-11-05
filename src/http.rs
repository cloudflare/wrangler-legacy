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
// todo: remove this and replace it entirely with cloudflare-rs
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

// This enum allows callers of format_error() to specify a help_*() function for adding additional
// detail to error messages, if desired.
pub enum ErrorCodeDetail {
    None,
    WorkersKV,
}

// Format errors from the cloudflare-rs cli for printing.
pub fn format_error(e: ApiFailure, err_type: ErrorCodeDetail) -> String {
    match e {
        ApiFailure::Error(status, api_errors) => {
            print_status_code_context(status);
            let mut complete_err = "".to_string();
            for error in api_errors.errors {
                let error_msg = format!("{} Code {}: {}\n", emoji::WARN, error.code, error.message);

                let suggestion = match err_type {
                    ErrorCodeDetail::WorkersKV => kv_help(error.code),
                    _ => "",
                };
                if !suggestion.is_empty() {
                    let help_msg = format!("{} {}\n", emoji::SLEUTH, suggestion);
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

// kv_help() provides more detailed explanations of Workers KV API error codes.
// See https://api.cloudflare.com/#workers-kv-namespace-errors for details.
fn kv_help(error_code: u16) -> &'static str {
    match error_code {
        7003 | 7000 => {
            "Your wrangler.toml is likely missing the field \"account_id\", which is required to write to Workers KV."
        }
        // namespace errors
        10010 | 10011 | 10012 | 10013 | 10014 | 10018 => {
            "Run `wrangler kv:namespace list` to see your existing namespaces with IDs"
        }
        10009 => "Run `wrangler kv:key list` to see your existing keys", // key errors
        // TODO: link to more info
        // limit errors
        10022 | 10024 | 10030 => "See documentation",
        // TODO: link to tool for this?
        // legacy namespace errors
        10021 | 10035 | 10038 => "Consider moving this namespace",
        // cloudflare account errors
        10017 | 10026 => "Workers KV is a paid feature, please upgrade your account (https://www.cloudflare.com/products/workers-kv/)",
        _ => "",
    }
}
