pub use std::convert::TryFrom;

#[derive(Clone, Copy)]
pub enum Protocol {
    Http,
    Https,
}

impl Protocol {
    pub fn is_http(self) -> bool {
        matches!(self, Protocol::Http)
    }

    pub fn is_https(self) -> bool {
        matches!(self, Protocol::Https)
    }
}

impl TryFrom<&str> for Protocol {
    type Error = failure::Error;

    fn try_from(p: &str) -> Result<Protocol, failure::Error> {
        match p {
            "http" => Ok(Protocol::Http),
            "https" => Ok(Protocol::Https),
            _ => failure::bail!("Invalid protocol, must be http or https"),
        }
    }
}
