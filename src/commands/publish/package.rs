use std::fs;
use std::path::Path;

use serde::{self, Deserialize};

#[derive(Debug, Deserialize)]
pub struct Package {
    #[serde(default)]
    main: String,
}
impl Package {
    pub fn main(&self) -> Result<String, failure::Error> {
        if self.main == "" {
            failure::bail!(
                "The `main` key in your `package.json` file is required; please specified the entrypoint of your Worker.",
            )
        } else if !Path::new(&self.main).exists() {
            failure::bail!(
                "The entrypoint of your Worker ({}) could not be found.",
                self.main
            )
        } else {
            Ok(self.main.clone())
        }
    }
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

        let package_json: String = fs::read_to_string(manifest_path.clone())?.parse()?;
        let package: Package = serde_json::from_str(&package_json)
            .expect(&format!("could not parse {:?}", manifest_path));

        Ok(package)
    }
}
