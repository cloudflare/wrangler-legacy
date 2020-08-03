use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Dev {
    pub host: Option<String>,
    pub ip: Option<String>,
    pub port: Option<u16>,
}
