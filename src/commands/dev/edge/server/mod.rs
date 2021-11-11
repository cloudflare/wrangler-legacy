mod http;
mod https;

pub use self::http::http;
pub use self::https::https;

use crate::commands::dev::utils::get_path_as_str;
use crate::commands::dev::Protocol;

use hyper::header::{HeaderName, HeaderValue};
use hyper::{Body, Request};

fn preview_request(
    mut parts: ::http::request::Parts,
    body: Body,
    preview_token: String,
    host: String,
    protocol: Protocol,
) -> Request<Body> {
    let path = get_path_as_str(&parts.uri);

    parts.headers.insert(
        HeaderName::from_static("host"),
        HeaderValue::from_str(&host).expect("Could not create host header"),
    );

    parts.headers.insert(
        HeaderName::from_static("cf-workers-preview-token"),
        HeaderValue::from_str(&preview_token).expect("Could not create token header"),
    );

    parts.uri = match protocol {
        Protocol::Http => format!("http://{}{}", host, path),
        Protocol::Https => format!("https://{}{}", host, path),
    }
    .parse()
    .expect("Could not construct preview url");

    Request::from_parts(parts, body)
}
