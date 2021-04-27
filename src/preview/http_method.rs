use anyhow::Result;
use std::str::FromStr;

#[derive(Clone, Debug)]
pub enum HttpMethod {
    Get,
    Post,
}

impl Default for HttpMethod {
    fn default() -> HttpMethod {
        HttpMethod::Get
    }
}

impl FromStr for HttpMethod {
    type Err = anyhow::Error;
    fn from_str(s: &str) -> Result<Self> {
        match s {
            "get" => Ok(HttpMethod::Get),
            "post" => Ok(HttpMethod::Post),
            _ => Ok(HttpMethod::default()),
        }
    }
}
