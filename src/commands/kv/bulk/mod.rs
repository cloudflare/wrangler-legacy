pub mod delete;
pub mod put;

pub use delete::run as delete;
pub use put::run as put;

use std::time::Duration;

use cloudflare::framework::auth::Credentials;
use cloudflare::framework::{Environment, HttpApiClient, HttpApiClientConfig};

use crate::http::feature::headers;
use crate::settings::global_user::GlobalUser;

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
