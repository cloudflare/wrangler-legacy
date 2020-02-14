pub(self) mod feature;
pub(self) mod legacy;
pub(self) mod v4;

pub use feature::Feature;
pub use legacy::{auth_client, client};
pub use v4::{cf_api_client, format_error, CfApiClientConfig};
