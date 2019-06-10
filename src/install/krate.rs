use crate::http;
use serde::{self, Deserialize};

#[derive(Debug, Deserialize)]
pub struct Krate {
    pub max_version: String,
}

#[derive(Debug, Deserialize)]
pub struct KrateResponse {
    #[serde(rename = "crate")]
    pub krate: Krate,
}

impl Krate {
    pub fn new(name: &str) -> Result<Krate, failure::Error> {
        let krate_address = format!("https://crates.io/api/v1/crates/{}", name);
        let client = http::client();
        let mut res = client.get(&krate_address).send()?;

        let kr: KrateResponse = serde_json::from_str(&res.text()?)?;
        Ok(kr.krate)
    }
}
