use std::str::FromStr;

pub enum HTTPMethod {
    Get,
    Post,
}

impl Default for HTTPMethod {
    fn default() -> HTTPMethod {
        HTTPMethod::Get
    }
}

impl FromStr for HTTPMethod {
    type Err = failure::Error;
    fn from_str(s: &str) -> Result<Self, failure::Error> {
        match s {
            "get" => Ok(HTTPMethod::Get),
            "post" => Ok(HTTPMethod::Post),
            _ => Ok(HTTPMethod::default()),
        }
    }
}
