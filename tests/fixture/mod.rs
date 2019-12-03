use std::collections::HashMap;
use std::env;
use std::fs;
use std::fs::File;
use std::io::prelude::*;
use std::path::PathBuf;
use std::process::Command;
use std::sync::Mutex;

use serde::Serialize;
use toml;

lazy_static! {
    static ref BUILD_LOCK: Mutex<u8> = Mutex::new(0);
}

const BUNDLE_OUT: &str = "./worker";

pub struct Fixture {
    name: String,
}

impl Fixture {
    pub fn new(name: &str) -> Fixture {
        let fixture = Fixture {
            name: name.to_string(),
        };

        let dest = fixture.get_path();

        if dest.exists() {
            fixture.cleanup();
        }

        fs::create_dir_all(dest.clone()).unwrap();
        fixture
    }

    pub fn scaffold_webpack(&self) {
        self.create_default_package_json();
        self.create_empty_js();
    }

    pub fn get_path(&self) -> PathBuf {
        let mut dest = env::temp_dir();
        dest.push(&self.name);
        dest
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

    pub fn cleanup(&self) {
        let path = self.get_path();
        assert!(path.exists(), format!("{:?} does not exist", path));

        // Workaround https://github.com/rust-lang/rust/issues/29497
        if cfg!(target_os = "windows") {
            let mut command = Command::new("cmd");
            command.arg("rmdir");
            command.arg("/s");
            command.arg(&path);
        } else {
            fs::remove_dir_all(&path).unwrap();
        }
    }
}

// small suite of flexible toml structs
// the  idea here is to focus on "when this config key is set"
// rather than needing to write tomls all the time.
// these structs set every value as an `Option`. To use,
// initialize a new WranglerToml::default() and begin setting
// values on it.
#[derive(Clone, Debug, Default, Serialize)]
pub struct KvConfig<'a> {
    pub binding: Option<&'a str>,
    pub id: Option<&'a str>,
}

#[derive(Clone, Debug, Default, Serialize)]
pub struct SiteConfig<'a> {
    pub bucket: Option<&'a str>,
    #[serde(rename = "entry-point")]
    pub entry_point: Option<&'a str>,
    pub include: Option<Vec<&'a str>>,
    pub exclude: Option<Vec<&'a str>>,
}

#[derive(Clone, Debug, Default, Serialize)]
pub struct EnvConfig<'a> {
    pub name: Option<&'a str>,
    pub account_id: Option<&'a str>,
    pub workers_dev: Option<bool>,
    pub route: Option<&'a str>,
    pub routes: Option<Vec<&'a str>>,
    pub zone_id: Option<&'a str>,
    pub webpack_config: Option<&'a str>,
    pub private: Option<bool>,
    pub site: Option<SiteConfig<'a>>,
    #[serde(rename = "kv-namespaces")]
    pub kv_namespaces: Option<Vec<KvConfig<'a>>>,
}

#[derive(Clone, Debug, Default, Serialize)]
pub struct WranglerToml<'a> {
    pub name: Option<&'a str>,
    #[serde(rename = "type")]
    pub target_type: Option<&'a str>,
    pub account_id: Option<&'a str>,
    pub workers_dev: Option<bool>,
    pub route: Option<&'a str>,
    pub routes: Option<Vec<&'a str>>,
    pub zone_id: Option<&'a str>,
    pub webpack_config: Option<&'a str>,
    pub private: Option<bool>,
    pub env: Option<HashMap<&'a str, EnvConfig<'a>>>,
    #[serde(rename = "kv-namespaces")]
    pub kv_namespaces: Option<Vec<KvConfig<'a>>>,
    pub site: Option<SiteConfig<'a>>,
}
