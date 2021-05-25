use crate::commands::dev::Protocol;
use serde::{Deserialize, Serialize};
use std::net::IpAddr;

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Dev {
    pub ip: Option<IpAddr>,
    pub port: Option<u16>,
    pub local_protocol: Option<Protocol>,
    pub upstream_protocol: Option<Protocol>,
}
