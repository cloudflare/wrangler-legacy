mod http;
mod https;

pub use self::http::http;
pub use self::https::https;

use crate::commands::dev::gcs::headers::structure_request;
use crate::commands::dev::utils::get_path_as_str;

use hyper::header::{HeaderName, HeaderValue};
use hyper::http::uri::InvalidUri;
use hyper::{Body, Request, Uri};

const PREVIEW_HOST: &str = "rawhttp.cloudflareworkers.com";

fn get_preview_url(path_string: &str) -> Result<Uri, InvalidUri> {
    format!("https://{}{}", PREVIEW_HOST, path_string).parse()
}

pub fn preview_request(
    mut parts: ::http::request::Parts,
    body: Body,
    preview_id: String,
) -> Request<Body> {
    let path = get_path_as_str(&parts.uri);
    let preview_id = &preview_id;

    structure_request(&mut parts);

    parts.headers.insert(
        HeaderName::from_static("host"),
        HeaderValue::from_static(PREVIEW_HOST),
    );

    parts.headers.insert(
        HeaderName::from_static("cf-ew-preview"),
        HeaderValue::from_str(preview_id).expect("Could not create header for preview id"),
    );

    parts.uri = get_preview_url(&path).expect("Could not get preview url");

    Request::from_parts(parts, body)
}
