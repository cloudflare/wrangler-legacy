use std::fs;
use std::path::{Path, PathBuf};

use anyhow::Result;
use serde::{self, Deserialize};

const PACKAGE_JSON_KEY_ERROR_MAIN: &str = "The `main` key in your `package.json` file is required; it must specify the entry point of your Worker.";
const PACKAGE_JSON_KEY_ERROR_MODULE: &str = "The `module` key in your `package.json` file is required when using the module script format; please specify the entry point of your Worker.";

#[derive(Debug, Deserialize)]
pub struct Package {
    #[serde(default)]
    main: PathBuf,
    #[serde(default)]
    module: PathBuf,
}
impl Package {
    pub fn main(&self, package_dir: &Path) -> Result<PathBuf> {
        if self.main == PathBuf::from("") {
            anyhow::bail!(PACKAGE_JSON_KEY_ERROR_MAIN,)
        } else if !package_dir.join(&self.main).exists() {
            anyhow::bail!(
                "The entrypoint of your Worker ({}) could not be found.",
                self.main.display()
            )
        } else {
            Ok(self.main.clone())
        }
    }
}

impl Package {
    pub fn new(package_dir: &Path) -> Result<Package> {
        let manifest_path = package_dir.join("package.json");
        if !manifest_path.is_file() {
            anyhow::bail!(
                "Your JavaScript project is missing a `package.json` file; is `{}` the \
                 wrong directory?",
                package_dir.display()
            )
        }

        let package_json: String = fs::read_to_string(manifest_path.clone())?.parse()?;

        serde_json::from_str(&package_json).map_err(|e| {
            anyhow::anyhow!(
                "could not parse {}, may have invalid or missing `main` or `module` keys: {}, \nHints:\n{}",
                manifest_path.display(),
                e,
                vec![PACKAGE_JSON_KEY_ERROR_MAIN, PACKAGE_JSON_KEY_ERROR_MODULE].join("\n"),
            )
        }) as Result<Package>
    }
}
