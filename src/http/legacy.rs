use reqwest::blocking::{Client, ClientBuilder};
use reqwest::header::{HeaderMap, HeaderValue};
use reqwest::redirect::Policy;
use std::time::Duration;

use crate::http::{feature::headers, Feature, DEFAULT_HTTP_TIMEOUT_SECONDS};
use crate::settings::global_user::GlobalUser;

// TODO: remove this and replace it entirely with cloudflare-rs
pub fn client() -> Client {
    builder()
        .default_headers(headers(None))
        .build()
        .expect("could not create http client")
}

pub fn legacy_auth_client(user: &GlobalUser) -> Client {
    get_client(user, None)
}

pub fn featured_legacy_auth_client(user: &GlobalUser, feature: Feature) -> Client {
    get_client(user, Some(feature))
}

fn get_client(user: &GlobalUser, feature: Option<Feature>) -> Client {
    let mut headers = headers(feature);
    add_auth_headers(&mut headers, user);

    builder()
        .default_headers(headers)
        .redirect(Policy::none())
        .build()
        .expect("could not create authenticated http client")
}

fn builder() -> ClientBuilder {
    let builder = reqwest::blocking::Client::builder();
    builder
        .connect_timeout(Duration::from_secs(10))
        .timeout(Duration::from_secs(DEFAULT_HTTP_TIMEOUT_SECONDS))
}

fn add_auth_headers(headers: &mut HeaderMap, user: &GlobalUser) {
    match user {
        GlobalUser::ApiTokenAuth { api_token } => {
            headers.insert(
                "Authorization",
                HeaderValue::from_str(&format!("Bearer {}", &api_token)).unwrap(),
            );
        }
        GlobalUser::OAuthTokenAuth {
            oauth_token,
            refresh_token: _,
            expiration_time: _,
        } => {
            headers.insert(
                "Authorization",
                HeaderValue::from_str(&format!("Bearer {}", &oauth_token)).unwrap(),
            );
        }
        GlobalUser::GlobalKeyAuth { email, api_key } => {
            headers.insert("X-Auth-Email", HeaderValue::from_str(email).unwrap());
            headers.insert("X-Auth-Key", HeaderValue::from_str(api_key).unwrap());
        }
    }
}
