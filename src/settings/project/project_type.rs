use std::fmt;
use std::str::FromStr;

use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum ProjectType {
    JavaScript,
    Rust,
    Webpack,
}

impl Default for ProjectType {
    fn default() -> Self {
        ProjectType::Webpack
    }
}

impl fmt::Display for ProjectType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let printable = match *self {
            ProjectType::JavaScript => "js",
            ProjectType::Rust => "rust",
            ProjectType::Webpack => "webpack",
        };
        write!(f, "{}", printable)
    }
}

impl FromStr for ProjectType {
    type Err = failure::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "javascript" => Ok(ProjectType::JavaScript),
            "rust" => Ok(ProjectType::Rust),
            "webpack" => Ok(ProjectType::Webpack),
            _ => failure::bail!("{} is not a valid wrangler project type!", s),
        }
    }
}
