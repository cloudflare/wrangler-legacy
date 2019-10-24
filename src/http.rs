use reqwest::header::{HeaderMap, HeaderValue, USER_AGENT};
use reqwest::{Client, ClientBuilder, RedirectPolicy};
use std::time::Duration;

use crate::install;
use crate::settings::global_user::GlobalUser;

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
    let auth_headers = add_auth_headers(&mut headers, user);

    builder()
        .default_headers(auth_headers.to_owned())
        .redirect(RedirectPolicy::none())
        .build()
        .expect("could not create authenticated http client")
}

fn add_auth_headers<'a>(headers: &'a mut HeaderMap, user: &GlobalUser) -> &'a HeaderMap {
    match &user.api_token {
        Some(token) => headers.insert(
            "Authorization",
            HeaderValue::from_str(&format!("Bearer {}", &token)).unwrap(),
        ),
        None => {
            // fallback to email + API key auth option
            match &user.email {
                Some(email) => {
                    headers.insert("X-Auth-Email", HeaderValue::from_str(&email).unwrap())
                }
                None => None,
            };
            match &user.api_key {
                Some(key) => headers.insert("X-Auth-Key", HeaderValue::from_str(&key).unwrap()),
                None => None,
            };
            None
        }
    };
    headers
}
