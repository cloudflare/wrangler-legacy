pub mod rust;
mod wrangler_toml;
pub use wrangler_toml::{EnvConfig, KvConfig, SiteConfig, WranglerToml};

use std::env;
use std::fs;
use std::fs::File;
use std::io::prelude::*;
use std::mem::ManuallyDrop;
use std::path::PathBuf;
use std::sync::MutexGuard;
use std::thread;

use tempfile::TempDir;
use toml;

const BUNDLE_OUT: &str = "worker";

pub struct Fixture<'a> {
    // we wrap the fixture's tempdir in a `ManuallyDrop` so that if a test
    // fails, its directory isn't deleted, and we have a chance to manually
    // inspect its state and figure out what is going on.
    dir: ManuallyDrop<TempDir>,
    output_path: &'a str,
}

impl Default for Fixture<'_> {
    fn default() -> Self {
        Self::new()
    }
}

impl Fixture<'_> {
    pub fn new() -> Fixture<'static> {
        let dir = TempDir::new().unwrap();
        eprintln!("Created fixture at {}", dir.path().display());
        Fixture {
            dir: ManuallyDrop::new(dir),
            output_path: BUNDLE_OUT,
        }
    }

    pub fn new_site() -> Fixture<'static> {
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
        fs::create_dir(&dir_path).unwrap();
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

    pub fn lock(&self) -> MutexGuard<'static, ()> {
        use std::sync::Mutex;

        lazy_static! {
            static ref ONE_TEST_AT_A_TIME: Mutex<()> = Mutex::new(());
        }

        ONE_TEST_AT_A_TIME.lock().unwrap_or_else(|e| e.into_inner())
    }

    pub fn set_config_home(&self) {
        // must set this environment variable for CI to pass
        let config_home_env_var = if cfg!(target_os = "windows") {
            "APPDATA"
        } else {
            "XDG_CONFIG_HOME"
        };
        let config_dir = self.get_path().join(".config");
        fs::create_dir(&config_dir).unwrap();
        env::set_var(config_home_env_var, config_dir.as_os_str());
    }
}

impl Drop for Fixture<'_> {
    fn drop(&mut self) {
        if !thread::panicking() {
            unsafe { ManuallyDrop::drop(&mut self.dir) }
        }
    }
}
