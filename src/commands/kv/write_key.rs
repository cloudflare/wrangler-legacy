// todo(gabbi): This file should use cloudflare-rs instead of our http::auth_client
// when https://github.com/cloudflare/cloudflare-rs/issues/26 is handled (this is
// because the SET key request body is not json--it is the raw value).

use std::fs;

use cloudflare::framework::response::ApiFailure;
use url::Url;

use crate::http;
use crate::settings::global_user::GlobalUser;
use crate::settings::project::Project;
use crate::terminal::message;

pub fn write_key(
    project: &Project,
    user: &GlobalUser,
    id: &str,
    key: &str,
    value: &str,
    is_file: bool,
    expiration: Option<&str>,
    expiration_ttl: Option<&str>,
) -> Result<(), failure::Error> {
    let api_endpoint = format!(
        "https://api.cloudflare.com/client/v4/accounts/{}/storage/kv/namespaces/{}/values/{}",
        project.account_id, id, key
    );

    // Add expiration, expiration_ttl, and base64 query options as necessary.
    let mut query_params: Vec<(&str, &str)> = vec![];
    match expiration {
        Some(exp) => query_params.push(("expiration", exp)),
        None => (),
    }
    match expiration_ttl {
        Some(ttl) => query_params.push(("expiration_ttl", ttl)),
        None => (),
    }
    let url = Url::parse_with_params(&api_endpoint, query_params);

    // If is_file is true, overwrite value to be the contents of the given
    // filename in the 'value' arg.
    let mut body_text: String;
    if is_file {
        body_text = fs::read_to_string(value)?;
    } else {
        body_text = value.to_string();
    }

    let client = http::auth_client(user);

    let url_into_str = url?.into_string();
    let mut res = client.put(&url_into_str).body(body_text).send()?;

    if res.status().is_success() {
        message::success("Success")
    } else {
        // This is logic pulled from cloudflare-rs for pretty error formatting right now;
        // it will be redundant when we switch to using cloudflare-rs for all API requests.
        let parsed = res.json();
        let errors = parsed.unwrap_or_default();
        super::print_error(ApiFailure::Error(res.status(), errors));
    }

    Ok(())
}
