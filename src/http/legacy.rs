use reqwest::blocking::{Client, ClientBuilder};
use reqwest::header::{HeaderMap, HeaderValue};
use reqwest::redirect::Policy;
use std::time::Duration;

use crate::http::{feature::headers, Feature};
use crate::settings::global_user::GlobalUser;

// TODO: remove this and replace it entirely with cloudflare-rs
pub fn client(feature: Option<Feature>) -> Client {
    builder()
        .default_headers(headers(feature))
        .build()
        .expect("could not create http client")
}

pub fn auth_client(feature: Option<Feature>, user: &GlobalUser) -> Client {
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
        .timeout(Duration::from_secs(30))
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
