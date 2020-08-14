pub use std::convert::TryFrom;

#[derive(Clone, Copy)]
pub enum Protocol {
    Http,
    Https,
}

impl Protocol {
    pub fn is_http(self) -> bool {
        match self {
            Protocol::Http => true,
            _ => false,
        }
    }

    pub fn is_https(self) -> bool {
        match self {
            Protocol::Https => true,
            _ => false,
        }
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
