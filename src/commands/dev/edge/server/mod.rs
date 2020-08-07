mod http;
mod https;

pub use self::http::http;
pub use self::https::https;

use crate::commands::dev::utils::get_path_as_str;

use hyper::client::{HttpConnector, ResponseFuture};
use hyper::header::{HeaderName, HeaderValue};
use hyper::{Body, Client as HyperClient, Request};
use hyper_rustls::HttpsConnector;

fn preview_request(
    req: Request<Body>,
    client: HyperClient<HttpsConnector<HttpConnector>>,
    preview_token: String,
    host: String,
    http: bool,
) -> ResponseFuture {
    let (mut parts, body) = req.into_parts();

    let path = get_path_as_str(&parts.uri);

    parts.headers.insert(
        HeaderName::from_static("host"),
        HeaderValue::from_str(&host).expect("Could not create host header"),
    );

    parts.headers.insert(
        HeaderName::from_static("cf-workers-preview-token"),
        HeaderValue::from_str(&preview_token).expect("Could not create token header"),
    );

    parts.uri = if http {
        format!("http://{}{}", host, path)
    } else {
        format!("https://{}{}", host, path)
    }
    .parse()
    .expect("Could not construct preview url");

    let req = Request::from_parts(parts, body);

    client.request(req)
}
