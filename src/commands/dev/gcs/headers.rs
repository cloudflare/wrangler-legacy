/// The preview service runs on cloudflareworkers.com and then sends parts
/// of the incoming request to the `request` in the Workers Runtime
///
/// The way it does this is by prefixing the _real_ headers sent to
/// and returned by the worker with this header prefix
const HEADER_PREFIX: &str = "cf-ew-raw-";
/// Any request headers sent to `wrangler dev` must be prefixed
/// before sending it to the preview service
/// and any response headers sent from the preview service
/// that don't have the prefix must be removed
/// and response headers that do have the prefix
/// must have the prefix stripped
use std::str::FromStr;

use anyhow::Result;
use hyper::header::{HeaderMap, HeaderName};
use hyper::http::request::Parts as RequestParts;
use hyper::http::response::Parts as ResponseParts;
use hyper::http::status::StatusCode;

/// modify an incoming request before sending it to the preview service
pub fn structure_request(parts: &mut RequestParts) {
    prepend_request_headers_prefix(parts)
}

/// modify a response from the preview service before returning it to the user
pub fn destructure_response(parts: &mut ResponseParts) -> Result<()> {
    set_response_status(parts)?;
    strip_response_headers_prefix(parts)
}

/// every header sent to `wrangler dev` must be prefixed
/// before sending it along to the preview service
/// so it is sent along to the Workers runtime and not
/// consumed directly by the preview service
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

/// every header returned by the Worker is prefixed
/// and headers without a prefix are specific to the preview service
/// here we parse those headers and construct a new header map
///
/// discard headers without that prefix
/// strip the prefix from real Workers headers
fn strip_response_headers_prefix(parts: &mut ResponseParts) -> Result<()> {
    let mut headers = HeaderMap::new();

    for header in &parts.headers {
        let (name, value) = header;
        let name = name.as_str();
        if let Some(header_name) = name.strip_prefix(HEADER_PREFIX) {
            let header_name = HeaderName::from_bytes(header_name.as_bytes())?;
            headers.append(header_name, value.clone());
        }
    }
    parts.headers = headers;
    Ok(())
}

/// parse the response status from headers sent by the preview service
/// and apply the parsed result to mutable ResponseParts
fn set_response_status(parts: &mut ResponseParts) -> Result<()> {
    let status = parts
        .headers
        .get("cf-ew-status")
        .expect("Could not determine status code of response");

    // `status` above will be "404 not found" or "200 ok"
    // we need to parse that string to create hyper's status code
    let status_vec: Vec<&str> = status.to_str()?.split(' ').collect();
    parts.status = StatusCode::from_str(status_vec[0])?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use hyper::Response;

    #[test]
    fn headers_are_appended() {
        let first_cookie = "chocolate chip".to_string();
        let second_cookie = "peanut butter".to_string();
        let response = Response::builder()
            .header("cf-ew-raw-Set-Cookie", &first_cookie)
            .header("cf-ew-raw-Set-Cookie", &second_cookie)
            .body(())
            .unwrap();
        let (mut parts, body) = response.into_parts();
        strip_response_headers_prefix(&mut parts).unwrap();
        let response = Response::from_parts(parts, body);
        let cookie_jar = response.headers().get_all("Set-Cookie");

        let mut iter = cookie_jar.iter();
        assert_eq!(&first_cookie, iter.next().unwrap());
        assert_eq!(&second_cookie, iter.next().unwrap());
        assert!(iter.next().is_none());
    }
}
