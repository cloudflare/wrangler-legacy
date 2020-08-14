use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Dev {
    pub ip: Option<String>,
    pub port: Option<u16>,
    pub local_protocol: Option<String>,
    pub upstream_protocol: Option<String>,
}
