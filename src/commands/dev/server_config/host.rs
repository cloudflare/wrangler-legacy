use std::fmt;

use anyhow::{anyhow, Result};
use url::Url;

#[derive(Debug, Clone)]
pub struct Host {
    url: Url,
    default: bool,
}

impl Host {
    pub fn new(host: &str, default: bool) -> Result<Self> {
        // try to create a url from host
        let url = match Url::parse(&host) {
            Ok(host) => Ok(host),
            // if it doesn't work, it might be because there was no scheme
            // default to https
            Err(_) => Url::parse(&format!("https://{}", host)),
        }?;

        // validate scheme
        let scheme = url.scheme();
        if scheme != "http" && scheme != "https" {
            anyhow::bail!("Your host scheme must be either http or https")
        }

        // validate host
        let host = url.host_str().ok_or_else(|| anyhow!("Invalid host, accepted formats are example.com, http://example.com, or https://example.com"))?;

        // recreate url without any trailing path
        let url = Url::parse(&format!("{}://{}", scheme, host))?;
        Ok(Host { url, default })
    }

    pub fn is_https(&self) -> bool {
        self.url.scheme() == "https"
    }

    pub fn is_default(&self) -> bool {
        self.default
    }
}

impl fmt::Display for Host {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.url.host_str().expect("could not parse host"))
    }
}
