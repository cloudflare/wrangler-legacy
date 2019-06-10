use reqwest::header::{HeaderMap, HeaderValue, USER_AGENT};
use reqwest::Client;

use crate::install;

fn client_headers() -> HeaderMap {
    let user_agent = if install::target::DEBUG {
        "wrangler/dev".to_string()
    } else {
        let version = env!("CARGO_PKG_VERSION");
        format!("wrangler/{}", version)
    };

    let value = HeaderValue::from_str(&user_agent).unwrap();
    let mut headers = HeaderMap::new();
    headers.insert(USER_AGENT, value);
    headers
}

pub fn client() -> Client {
    let builder = reqwest::Client::builder();
    builder
        .default_headers(client_headers())
        .build()
        .expect("could not create http client")
}
