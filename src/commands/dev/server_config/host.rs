use std::fmt;

use failure::format_err;
use url::Url;

#[derive(Clone)]
pub struct Host {
    url: Url,
}

impl Host {
    pub fn new(host: &str) -> Result<Self, failure::Error> {
        let url = match Url::parse(&host) {
            Ok(host) => Ok(host),
            Err(_) => Url::parse(&format!("https://{}", host)),
        }?;

        let scheme = url.scheme();
        if scheme != "http" && scheme != "https" {
            failure::bail!("Your host scheme must be either http or https")
        }

        let host = url.host_str().ok_or(format_err!("Invalid host, accepted formats are example.com, http://example.com, or https://example.com"))?;
        let url = Url::parse(&format!("{}://{}", scheme, host))?;
        Ok(Host { url })
    }

    pub fn is_https(&self) -> bool {
        self.url.scheme() == "https"
    }
}

impl fmt::Display for Host {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.url.host_str().unwrap().to_string())
    }
}
