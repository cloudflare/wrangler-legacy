use std::fmt;
use std::net::{SocketAddr, ToSocketAddrs};

use failure::format_err;

#[derive(Debug, Clone)]
pub struct ListeningAddress {
    pub address: SocketAddr,
}

impl ListeningAddress {
    pub fn new(ip: &str, port: &str) -> Result<Self, failure::Error> {
        let address = format!("{}:{}", ip, port);
        let mut address_iter = address.to_socket_addrs()?;
        let address = address_iter
            .next()
            .ok_or_else(|| format_err!("Could not parse address {}", address))?;
        Ok(ListeningAddress { address })
    }

    fn as_str(&self) -> String {
        self.address.to_string().replace("[::1]", "localhost")
    }
}

impl fmt::Display for ListeningAddress {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}
