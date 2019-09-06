// todo(gabbi): This file should use cloudflare-rs instead of our http::auth_client
// when https://github.com/cloudflare/cloudflare-rs/issues/26 is handled (this is
// because the SET key request body is not json--it is the raw value).

use cloudflare::framework::response::ApiFailure;
use url::Url;

use crate::commands::kv;
use crate::http;
use crate::settings::global_user::GlobalUser;
use crate::settings::project::Project;
use crate::terminal::message;

pub fn put(
    project: &Project,
    user: GlobalUser,
    id: &str,
    key: &str,
    value: String,
    expiration: Option<&str>,
    expiration_ttl: Option<&str>,
) -> Result<(), failure::Error> {
    let api_endpoint = format!(
        "https://api.cloudflare.com/client/v4/accounts/{}/storage/kv/namespaces/{}/values/{}",
        project.account_id,
        id,
        kv::url_encode_key(key)
    );

    // Add expiration and expiration_ttl query options as necessary.
    let mut query_params: Vec<(&str, &str)> = vec![];

    if let Some(exp) = expiration {
        query_params.push(("expiration", exp))
    }

    if let Some(ttl) = expiration_ttl {
        query_params.push(("expiration_ttl", ttl))
    }

    let url = Url::parse_with_params(&api_endpoint, query_params);

    let client = http::auth_client(&user);

    let url_into_str = url?.into_string();
    let mut res = client.put(&url_into_str).body(value).send()?;

    if res.status().is_success() {
        message::success("Success")
    } else {
        // This is logic pulled from cloudflare-rs for pretty error formatting right now;
        // it will be redundant when we switch to using cloudflare-rs for all API requests.
        let parsed = res.json();
        let errors = parsed.unwrap_or_default();
        kv::print_error(ApiFailure::Error(res.status(), errors));
    }

    Ok(())
}
