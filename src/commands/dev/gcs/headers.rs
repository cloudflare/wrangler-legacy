const HEADER_PREFIX: &str = "cf-ew-raw-";

use std::str::FromStr;

use hyper::header::{HeaderMap, HeaderName};
use hyper::http::request::Parts as RequestParts;
use hyper::http::response::Parts as ResponseParts;
use hyper::http::status::StatusCode;

pub fn structure_request(parts: &mut RequestParts) {
    prepend_request_headers_prefix(parts)
}

pub fn destructure_response(parts: &mut ResponseParts) -> Result<(), failure::Error> {
    set_response_status(parts)?;
    strip_response_headers_prefix(parts)
}

fn prepend_request_headers_prefix(parts: &mut RequestParts) {
    let mut headers: HeaderMap = HeaderMap::new();

    for header in &parts.headers {
        let (name, value) = header;
        let forward_header = format!("{}{}", HEADER_PREFIX, name);
        let header_name = HeaderName::from_bytes(forward_header.as_bytes())
            .unwrap_or_else(|_| panic!("Could not create header name for {}", name));
        headers.insert(header_name, value.clone());
    }
    parts.headers = headers;
}

fn strip_response_headers_prefix(parts: &mut ResponseParts) -> Result<(), failure::Error> {
    let mut headers = HeaderMap::new();

    for header in &parts.headers {
        let (name, value) = header;
        let name = name.as_str();
        if name.starts_with(HEADER_PREFIX) {
            let header_name = &name[HEADER_PREFIX.len()..];
            let header_name = HeaderName::from_bytes(header_name.as_bytes())?;
            headers.insert(header_name, value.clone());
        }
    }
    parts.headers = headers;
    Ok(())
}

fn set_response_status(parts: &mut ResponseParts) -> Result<(), failure::Error> {
    let status = parts
        .headers
        .get("cf-ew-status")
        .expect("Could not determine status code of response");
    // status will be "404 not found" or "200 ok"
    // we need to split that string to create hyper's status code
    let status_vec: Vec<&str> = status.to_str()?.split(' ').collect();
    parts.status = StatusCode::from_str(status_vec[0])?;
    Ok(())
}
