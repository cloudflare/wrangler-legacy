mod host;
mod listening_address;

use host::Host;
use listening_address::ListeningAddress;

#[derive(Debug, Clone)]
pub struct ServerConfig {
    pub host: Host,
    pub listening_address: ListeningAddress,
}

impl ServerConfig {
    pub fn new(
        host: Option<&str>,
        ip: Option<&str>,
        port: Option<&str>,
    ) -> Result<Self, failure::Error> {
        let port = port.unwrap_or("8787");
        let ip = ip.unwrap_or("localhost");
        let host = host.unwrap_or("https://example.com").to_string();

        let listening_address = ListeningAddress::new(ip, port)?;
        let host = Host::new(&host)?;

        Ok(ServerConfig {
            host,
            listening_address,
        })
    }
}
