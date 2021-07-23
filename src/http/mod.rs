pub(self) mod cf;
pub(crate) mod feature;
pub(self) mod legacy;

pub const DEFAULT_HTTP_TIMEOUT_SECONDS: u64 = 60;
pub use cf::{cf_v4_api_client_async, cf_v4_client, format_error};
pub use feature::Feature;
pub use legacy::{client, featured_legacy_auth_client, legacy_auth_client};
