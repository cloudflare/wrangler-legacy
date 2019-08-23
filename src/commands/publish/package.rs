use std::fs;
use std::path::{Path, PathBuf};

use serde::{self, Deserialize};

pub struct Package {
    main: PathBuf,
}

#[derive(Debug, Deserialize)]
struct PackageRaw {
    #[serde(default)]
    main: String,
}

impl Package {
    pub fn main(&self) -> Result<&Path, failure::Error> {
        if !self.main.exists() {
            failure::bail!(
                "The entrypoint of your Worker ({}) could not be found.",
                self.main.display()
            )
        } else {
            Ok(self.main.as_ref())
        }
    }
}

impl Package {
    pub fn new(pkg_path: &Path) -> Result<Package, failure::Error> {
        let manifest_path = pkg_path.join("package.json");
        if !manifest_path.is_file() {
            failure::bail!(
                "Your JavaScript project is missing a `package.json` file; is `{}` the \
                 wrong directory?",
                pkg_path.display()
            )
        }

        let package_json: String = fs::read_to_string(manifest_path.clone())?.parse()?;
        let package_raw: PackageRaw = serde_json::from_str(&package_json)
            .unwrap_or_else(|_| panic!("could not parse {:?}", manifest_path));

        if package_raw.main == "" {
            failure::bail!(
                "The `main` key in your `package.json` file is required; please specified the entrypoint of your Worker.",
            );
        }

        let package = Package {
            main: pkg_path.join(package_raw.main),
        };

        Ok(package)
    }
}
