mod host;
mod listening_address;

use host::Host;
use listening_address::ListeningAddress;

use http::Uri;

#[derive(Debug, Clone)]
pub struct ServerConfig {
    pub host: Host,
    pub listening_address: ListeningAddress,
    pub allowed_origins: Vec<Uri>,
}

impl ServerConfig {
    pub fn new(
        host: Option<&str>,
        ip: Option<&str>,
        port: Option<&str>,
        allowed_origins: &[&str],
    ) -> Result<Self, failure::Error> {
        let port = port.unwrap_or("8787");
        let ip = ip.unwrap_or("localhost");
        let host = host.unwrap_or("https://example.com").to_string();
        let allowed_origins = allowed_origins
            .into_iter()
            .map(|o| o.parse::<Uri>().expect("failed to parse allowed origin"))
            .collect();
        let listening_address = ListeningAddress::new(ip, port)?;
        let host = Host::new(&host)?;

        Ok(ServerConfig {
            host,
            listening_address,
            allowed_origins,
        })
    }
}
