use std::fmt;
use std::str::FromStr;

use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum TargetType {
    JavaScript,
    Rust,
    Webpack,
}

impl Default for TargetType {
    fn default() -> Self {
        TargetType::Webpack
    }
}

impl fmt::Display for TargetType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let printable = match *self {
            TargetType::JavaScript => "js",
            TargetType::Rust => "rust",
            TargetType::Webpack => "webpack",
        };
        write!(f, "{}", printable)
    }
}

impl FromStr for TargetType {
    type Err = failure::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "javascript" => Ok(TargetType::JavaScript),
            "rust" => Ok(TargetType::Rust),
            "webpack" => Ok(TargetType::Webpack),
            _ => failure::bail!("{} is not a valid wrangler build type!", s),
        }
    }
}
