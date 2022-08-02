use std::fs;
use std::path::Path;

use anyhow::Result;
use serde::{self, Deserialize};
use toml_edit::easy as toml;

#[derive(Debug, Deserialize)]
pub struct Krate {
    pub name: String,
}

#[derive(Debug, Deserialize)]
struct KrateManifest {
    pub package: Krate,
}

impl Krate {
    pub fn new(krate_path: &str) -> Result<Krate> {
        let manifest_path = Path::new(krate_path).join("Cargo.toml");
        if !manifest_path.is_file() {
            anyhow::bail!(
                "crate directory is missing a `Cargo.toml` file; is `{}` the \
                 wrong directory?",
                krate_path
            )
        }

        let cargo_toml: String = fs::read_to_string(manifest_path)?.parse()?;
        let krate: KrateManifest = toml::from_str(&cargo_toml)?;

        Ok(krate.package)
    }
}
