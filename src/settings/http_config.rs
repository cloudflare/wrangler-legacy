use serde::{Deserialize, Serialize};
use std::{str::FromStr, time::Duration};

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
pub struct HttpConfig {
    connect_timeout: Option<u64>,
    http_timeout: Option<u64>,
    bulk_timeout: Option<u64>,
}

impl HttpConfig {
    pub fn get_connect_timeout(&self) -> Duration {
        Duration::from_secs(
            self.connect_timeout
                .unwrap_or(crate::http::DEFAULT_CONNECT_TIMEOUT_SECONDS),
        )
    }

    pub fn get_http_timeout(&self) -> Duration {
        Duration::from_secs(
            self.http_timeout
                .unwrap_or(crate::http::DEFAULT_HTTP_TIMEOUT_SECONDS),
        )
    }

    pub fn get_bulk_timeout(&self) -> Duration {
        Duration::from_secs(
            self.bulk_timeout
                .unwrap_or(crate::http::DEFAULT_BULK_TIMEOUT_SECONDS),
        )
    }
}

impl FromStr for HttpConfig {
    type Err = toml::de::Error;

    fn from_str(serialized_toml: &str) -> Result<Self, Self::Err> {
        toml::from_str(serialized_toml)
    }
}

impl Default for HttpConfig {
    fn default() -> Self {
        HttpConfig {
            connect_timeout: None,
            http_timeout: None,
            bulk_timeout: None,
        }
    }
}

#[test]
fn it_reads_timeout_when_specified_otherwise_reads_default_values() {
    // make sure that the values in our test aren't the default values,
    // wouln't want this test to break if the defaults happen to be the same
    // as the test
    assert_ne!(3, crate::http::DEFAULT_CONNECT_TIMEOUT_SECONDS);
    assert_ne!(8, crate::http::DEFAULT_HTTP_TIMEOUT_SECONDS);
    assert_ne!(16, crate::http::DEFAULT_BULK_TIMEOUT_SECONDS);

    // assert that reading a config yields the timeouts specified
    let http_config = HttpConfig::from_str(
        r#"
connect_timeout = 3
http_timeout = 8
bulk_timeout = 16
"#,
    )
    .unwrap();

    assert_eq!(Duration::from_secs(3), http_config.get_connect_timeout());
    assert_eq!(Duration::from_secs(8), http_config.get_http_timeout());
    assert_eq!(Duration::from_secs(16), http_config.get_bulk_timeout());

    // assert that a lack of a config uses the defaults
    let manifest = HttpConfig::from_str("").unwrap();

    assert_eq!(
        Duration::from_secs(crate::http::DEFAULT_CONNECT_TIMEOUT_SECONDS),
        manifest.get_connect_timeout()
    );
    assert_eq!(
        Duration::from_secs(crate::http::DEFAULT_HTTP_TIMEOUT_SECONDS),
        manifest.get_http_timeout()
    );
    assert_eq!(
        Duration::from_secs(crate::http::DEFAULT_BULK_TIMEOUT_SECONDS),
        manifest.get_bulk_timeout()
    );
}
