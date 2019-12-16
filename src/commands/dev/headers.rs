const HEADER_PREFIX: &str = "cf-ew-raw-";

use hyper::header::{HeaderMap, HeaderName};
use hyper::http::request::Parts as RequestParts;
use hyper::http::response::Parts as ResponseParts;

pub fn strip_response_headers_prefix(parts: &mut ResponseParts) -> Result<(), failure::Error> {
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

pub fn prepend_request_headers_prefix(parts: &mut RequestParts) {
    let mut headers: HeaderMap = HeaderMap::new();

    for header in &parts.headers {
        let (name, value) = header;
        let forward_header = format!("{}{}", HEADER_PREFIX, name);
        let header_name = HeaderName::from_bytes(forward_header.as_bytes())
            .expect(&format!("Could not create header name for {}", name));
        headers.insert(header_name, value.clone());
    }
    parts.headers = headers;
}
