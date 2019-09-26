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
    headers.insert("X-Auth-Key", HeaderValue::from_str(&user.api_key).unwrap());
    headers.insert("X-Auth-Email", HeaderValue::from_str(&user.email).unwrap());

    builder()
        .default_headers(headers)
        .redirect(RedirectPolicy::none())
        .build()
        .expect("could not create authenticated http client")
}
