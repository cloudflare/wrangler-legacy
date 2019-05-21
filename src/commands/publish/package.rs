use std::fs;
use std::path::Path;

use serde::{self, Deserialize};

#[derive(Debug, Deserialize)]
pub struct Package {
    pub main: String,
}

impl Package {
    pub fn new(pkg_path: &str) -> Result<Package, failure::Error> {
        let manifest_path = Path::new(pkg_path).join("package.json");
        if !manifest_path.is_file() {
            failure::bail!(
                "Your JavaScript project is missing a `package.json` file; is `{}` the \
                 wrong directory?",
                pkg_path
            )
        }

        let package_json: String = fs::read_to_string(manifest_path)?.parse()?;
        let package: Package = serde_json::from_str(&package_json)?;

        Ok(package)
    }
}
