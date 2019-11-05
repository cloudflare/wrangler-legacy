use std::env;
use std::path::PathBuf;

use serde::{Deserialize, Serialize};

const SITE_ENTRY_POINT: &str = "workers-site";

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Site {
    pub bucket: String,
    #[serde(rename = "entry-point")]
    entry_point: Option<PathBuf>,
    pub include: Option<Vec<String>>,
    pub exclude: Option<Vec<String>>,
}

impl Site {
    pub fn new(bucket: &str) -> Site {
        let mut site = Site::default();
        site.bucket = String::from(bucket);

        site
    }

    // if the user has configured `site.entry-point`, use that
    // as the build directory. Otherwise use the default const
    // SITE_ENTRY_POINT
    pub fn entry_point(&self) -> Result<PathBuf, std::io::Error> {
        let current_dir = env::current_dir()?;
        Ok(current_dir.join(
            self.entry_point
                .to_owned()
                .unwrap_or_else(|| PathBuf::from(SITE_ENTRY_POINT)),
        ))
    }
}

impl Default for Site {
    fn default() -> Site {
        Site {
            bucket: String::new(),
            entry_point: Some(PathBuf::from(SITE_ENTRY_POINT)),
            include: None,
            exclude: None,
        }
    }
}
