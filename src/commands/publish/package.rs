use std::fs;
use std::path::PathBuf;

use serde::{self, Deserialize};

#[derive(Debug, Deserialize)]
pub struct Package {
    #[serde(default)]
    main: PathBuf,
}
impl Package {
    pub fn main(&self, build_dir: &PathBuf) -> Result<PathBuf, failure::Error> {
        if self.main == PathBuf::from("") {
            failure::bail!(
                "The `main` key in your `package.json` file is required; please specified the entrypoint of your Worker.",
            )
        } else if !build_dir.join(&self.main).exists() {
            failure::bail!(
                "The entrypoint of your Worker ({:?}) could not be found.",
                self.main
            )
        } else {
            Ok(self.main.clone())
        }
    }
}

impl Package {
    pub fn new(pkg_path: &PathBuf) -> Result<Package, failure::Error> {
        let manifest_path = pkg_path.join("package.json");
        if !manifest_path.is_file() {
            failure::bail!(
                "Your JavaScript project is missing a `package.json` file; is `{:?}` the \
                 wrong directory?",
                pkg_path
            )
        }

        let package_json: String = fs::read_to_string(manifest_path.clone())?.parse()?;
        let package: Package = serde_json::from_str(&package_json)
            .unwrap_or_else(|_| panic!("could not parse {:?}", manifest_path));

        Ok(package)
    }
}
