mod wrangler_toml;
pub use wrangler_toml::{EnvConfig, KvConfig, SiteConfig, WranglerToml};

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

pub struct Fixture {
    // we wrap the fixture's tempdir in a `ManuallyDrop` so that if a test
    // fails, its directory isn't deleted, and we have a chance to manually
    // inspect its state and figure out what is going on.
    dir: ManuallyDrop<TempDir>,
}

impl Fixture {
    pub fn new() -> Fixture {
        let dir = TempDir::new().unwrap();
        eprintln!("Created fixture at {}", dir.path().display());
        Fixture {
            dir: ManuallyDrop::new(dir),
        }
    }

    pub fn get_path(&self) -> PathBuf {
        self.dir.path().to_path_buf()
    }

    pub fn scaffold_webpack(&self) {
        self.create_default_package_json();
        self.create_empty_js();
    }

    pub fn get_output_path(&self) -> PathBuf {
        self.get_path().join(BUNDLE_OUT)
    }

    pub fn create_file(&self, name: &str, content: &str) {
        let file_path = self.get_path().join(name);
        let mut file = File::create(file_path).unwrap();
        let content = String::from(content);
        file.write_all(content.as_bytes()).unwrap();
    }

    pub fn create_dir(&self, name: &str) {
        let dir_path = self.get_path().join(name);
        fs::remove_dir_all(&dir_path).ok();
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

    pub fn lock(&self) -> MutexGuard<'static, ()> {
        use std::sync::Mutex;

        lazy_static! {
            static ref ONE_TEST_AT_A_TIME: Mutex<()> = Mutex::new(());
        }

        ONE_TEST_AT_A_TIME.lock().unwrap_or_else(|e| e.into_inner())
    }
}

impl Drop for Fixture {
    fn drop(&mut self) {
        if !thread::panicking() {
            unsafe { ManuallyDrop::drop(&mut self.dir) }
        }
    }
}
