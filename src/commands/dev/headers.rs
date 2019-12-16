use crate::commands::dev::server_config::ServerConfig;

const HEADER_PREFIX: &str = "cf-ew-raw-";

use hyper::header::{HeaderMap, HeaderName, HeaderValue};
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

pub fn prepend_request_headers_prefix(parts: &mut RequestParts, server_config: &ServerConfig) {
    let mut headers: HeaderMap = HeaderMap::new();

    for header in &parts.headers {
        let (name, value) = header;
        let value = if name == "host" {
            let dev_server_host = server_config.listening_address.to_string();
            let backend_host = server_config.host.to_string();
            let new_value = value
                .clone()
                .to_str()
                .expect("Could not parse incoming Host header")
                .to_string()
                .replace(&dev_server_host, &backend_host);
            HeaderValue::from_bytes(new_value.as_bytes())
                .expect("Could not rewrite incoming Host header")
        } else {
            value.clone()
        };
        let forward_header = format!("{}{}", HEADER_PREFIX, name);
        let header_name = HeaderName::from_bytes(forward_header.as_bytes())
            .unwrap_or_else(|_| panic!("Could not create header name for {}", name));
        headers.insert(header_name, value);
    }
    parts.headers = headers;
}
