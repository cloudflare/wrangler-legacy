use std::path::PathBuf;

use serde::{Deserialize, Serialize};

const SITE_ENTRY_POINT: &str = "workers-site";

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Site {
    pub bucket: String,
    #[serde(rename = "entry-point")]
    pub entry_point: Option<String>,
    pub include: Option<Vec<String>>,
    pub exclude: Option<Vec<String>>,
}

impl Site {
    pub fn new(bucket: &str) -> Site {
        let mut site = Site::default();
        site.bucket = String::from(bucket);

        site
    }

    pub fn build_dir(&self, current_dir: PathBuf) -> Result<PathBuf, std::io::Error> {
        Ok(current_dir.join(
            self.entry_point
                .to_owned()
                .unwrap_or_else(|| format!("./{}", SITE_ENTRY_POINT)),
        ))
    }
}

impl Default for Site {
    fn default() -> Site {
        Site {
            bucket: String::new(),
            entry_point: Some(String::from(SITE_ENTRY_POINT)),
            include: None,
            exclude: None,
        }
    }
}
