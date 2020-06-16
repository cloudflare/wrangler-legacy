use std::time::Duration;

use cloudflare::endpoints::workerskv::delete_bulk::DeleteBulk;
use cloudflare::endpoints::workerskv::write_bulk::KeyValuePair;
use cloudflare::endpoints::workerskv::write_bulk::WriteBulk;
use cloudflare::framework::apiclient::ApiClient;
use cloudflare::framework::auth::Credentials;
use cloudflare::framework::{Environment, HttpApiClient, HttpApiClientConfig};

use crate::commands::kv::format_error;
use crate::http::feature::headers;
use crate::settings::global_user::GlobalUser;
use crate::settings::toml::Target;

pub const MAX_PAIRS: usize = 10000;

// Create a special API client that has a longer timeout than usual, given that KV operations
// can be lengthy if payloads are large.
fn bulk_api_client(user: &GlobalUser) -> Result<HttpApiClient, failure::Error> {
    let config = HttpApiClientConfig {
        http_timeout: Duration::from_secs(5 * 60),
        default_headers: headers(None),
    };

    HttpApiClient::new(
        Credentials::from(user.to_owned()),
        config,
        Environment::Production,
    )
}

pub fn put(
    target: &Target,
    user: &GlobalUser,
    namespace_id: &str,
    pairs: &[KeyValuePair],
) -> Result<(), failure::Error> {
    let client = bulk_api_client(user)?;

    match client.request(&WriteBulk {
        account_identifier: &target.account_id,
        namespace_identifier: namespace_id,
        bulk_key_value_pairs: pairs.to_owned(),
    }) {
        Ok(_) => Ok(()),
        Err(e) => failure::bail!("{}", format_error(e)),
    }
}

pub fn delete(
    target: &Target,
    user: &GlobalUser,
    namespace_id: &str,
    keys: Vec<String>,
) -> Result<(), failure::Error> {
    let client = bulk_api_client(user)?;

    let response = client.request(&DeleteBulk {
        account_identifier: &target.account_id,
        namespace_identifier: namespace_id,
        bulk_keys: keys,
    });

    match response {
        Ok(_) => Ok(()),
        Err(e) => failure::bail!("{}", format_error(e)),
    }
}
