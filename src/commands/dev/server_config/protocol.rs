use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use std::{convert::TryFrom, str::FromStr};

#[derive(Debug, PartialEq, Eq, Clone, Copy, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
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
    type Error = <Self as FromStr>::Err;

    fn try_from(p: &str) -> Result<Protocol> {
        p.parse()
    }
}

impl FromStr for Protocol {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        match s {
            "http" => Ok(Protocol::Http),
            "https" => Ok(Protocol::Https),
            _ => Err(anyhow!("Invalid protocol, must be http or https")),
        }
    }
}
