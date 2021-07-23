// This is used from two different integration tests and a unit test, which
// makes the compiler confused about what code is used and what isn't.
#![allow(dead_code)]

mod wrangler_toml;
pub use wrangler_toml::{EnvConfig, KvConfig, SiteConfig, Triggers, WranglerToml, TEST_ENV_NAME};

use std::fs;
use std::fs::File;
use std::io::prelude::*;
use std::mem::ManuallyDrop;
use std::path::PathBuf;
use std::thread;

use tempfile::TempDir;

const BUNDLE_OUT: &str = "worker";

pub struct Fixture {
    // we wrap the fixture's tempdir in a `ManuallyDrop` so that if a test
    // fails, its directory isn't deleted, and we have a chance to manually
    // inspect its state and figure out what is going on.
    dir: ManuallyDrop<TempDir>,
    output_path: &'static str,
}

impl Default for Fixture {
    fn default() -> Self {
        Self::new()
    }
}

impl Fixture {
    pub fn new() -> Fixture {
        let dir = TempDir::new().unwrap();
        eprintln!("Created fixture at {}", dir.path().display());
        Fixture {
            dir: ManuallyDrop::new(dir),
            output_path: BUNDLE_OUT,
        }
    }

    pub fn new_site() -> Fixture {
        let mut fixture = Fixture::new();
        fixture.output_path = "workers-site/worker";

        fixture.scaffold_site();

        fixture
    }

    pub fn get_path(&self) -> PathBuf {
        self.dir.path().to_path_buf()
    }

    pub fn scaffold_webpack(&self) {
        self.create_default_package_json();
        self.create_empty_js();
    }

    pub fn get_output_path(&self) -> PathBuf {
        self.get_path().join(self.output_path)
    }

    pub fn create_file(&self, name: &str, content: &str) {
        let file_path = self.get_path().join(name);
        let mut file = File::create(file_path).unwrap();
        let content = String::from(content);
        file.write_all(content.as_bytes()).unwrap();
    }

    pub fn create_dir(&self, name: &str) {
        let dir_path = self.get_path().join(name);
        fs::create_dir(dir_path).unwrap();
    }

    pub fn create_empty_js(&self) {
        self.create_file("index.js", "");
    }

    pub fn create_default_package_json(&self) {
        self.create_file(
            "package.json",
            r#"
            {
                "main": "index.js"
            }
        "#,
        );
    }

    pub fn create_wrangler_toml(&self, wrangler_toml: WranglerToml) {
        self.create_file("wrangler.toml", &toml::to_string(&wrangler_toml).unwrap());
    }

    pub fn scaffold_site(&self) {
        self.create_dir("workers-site");
        self.create_file(
            "workers-site/package.json",
            r#"
            {
              "private": true,
              "main": "index.js",
              "dependencies": {
                "@cloudflare/kv-asset-handler": "^0.0.5"
              }
            }
        "#,
        );
        self.create_file("workers-site/index.js", "");
    }
}

impl Drop for Fixture {
    fn drop(&mut self) {
        if !thread::panicking() {
            unsafe { ManuallyDrop::drop(&mut self.dir) }
        }
    }
}
