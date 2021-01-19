use std::fmt;
use std::str::FromStr;

use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
pub enum ScriptFormat {
    #[serde(rename = "service-worker")]
    ServiceWorker,
    #[serde(rename = "modules")]
    Modules,
}

impl fmt::Display for ScriptFormat {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let printable = match *self {
            Self::ServiceWorker => "service-worker",
            Self::Modules => "modules",
        };
        write!(f, "{}", printable)
    }
}

impl FromStr for ScriptFormat {
    type Err = failure::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "service-worker" => Ok(Self::ServiceWorker),
            "modules" => Ok(Self::Modules),
            _ => failure::bail!("{} is not a valid script format!", s),
        }
    }
}
