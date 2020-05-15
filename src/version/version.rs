use std::fs;
use std::str::FromStr;
use std::sync::mpsc;
use std::thread;
use std::time::SystemTime;

use crate::settings::global_user::get_wrangler_home_dir;

use reqwest::header::USER_AGENT;
use semver::Version;
use serde::{Deserialize, Serialize};

const ONE_DAY: u64 = 60 * 60 * 24;

#[derive(Debug)]
pub struct WranglerVersion {
    /// currently installed version of wrangler
    pub current: Version,

    /// latest version of wrangler on crates.io
    pub latest: Version,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
struct LastCheckedVersion {
    /// latest version as of last time we checked
    latest_version: String,

    /// the last time we asked crates.io for the latest version
    last_checked: SystemTime,
}

impl FromStr for LastCheckedVersion {
    type Err = toml::de::Error;

    fn from_str(serialized_toml: &str) -> Result<Self, Self::Err> {
        toml::from_str(serialized_toml)
    }
}

fn get_installed_version() -> Result<Version, failure::Error> {
    let version = option_env!("CARGO_PKG_VERSION").unwrap_or_else(|| "unknown");
    let parsed_version = Version::parse(version)?;
    Ok(parsed_version)
}

fn check_wrangler_versions() -> Result<WranglerVersion, failure::Error> {
    let current = get_installed_version()?;
    let latest = get_latest_version(&current.to_string())?;
    Ok(WranglerVersion { current, latest })
}

fn get_latest_version(installed_version: &str) -> Result<Version, failure::Error> {
    let config_dir = get_wrangler_home_dir()?;
    let version_file = config_dir.join("version.toml");
    let current_time = SystemTime::now();
    let latest_version = match fs::read_to_string(&version_file) {
        Ok(contents) => {
            let last_checked_version = LastCheckedVersion::from_str(&contents)?;
            let time_since_last_checked =
                current_time.duration_since(last_checked_version.last_checked)?;
            if time_since_last_checked.as_secs() < ONE_DAY {
                let version = Version::parse(&last_checked_version.latest_version)?;
                Some(version)
            } else {
                None
            }
        }
        Err(_) => None,
    };
    match latest_version {
        Some(latest_version) => Ok(latest_version),
        None => {
            let latest_version = get_latest_version_from_api(installed_version)?;
            let updated_file_contents = toml::to_string(&LastCheckedVersion {
                latest_version: latest_version.to_string(),
                last_checked: current_time,
            })?;
            fs::write(&version_file, updated_file_contents)?;
            Ok(latest_version)
        }
    }
}

fn get_latest_version_from_api(installed_version: &str) -> Result<Version, failure::Error> {
    let url = "https://crates.io/api/v1/crates/wrangler";
    let user_agent = format!(
        "wrangler/{} ({})",
        installed_version,
        env!("CARGO_PKG_REPOSITORY")
    );
    let response = reqwest::blocking::Client::new()
        .get(url)
        .header(USER_AGENT, user_agent)
        .send()?
        .error_for_status()?;
    let text = response.text()?;
    let crt: ApiResponse = serde_json::from_str(&text)?;
    let version = Version::parse(&crt.info.max_version)?;
    Ok(version)
}

#[derive(Deserialize, Debug)]
struct ApiResponse {
    #[serde(rename = "crate")]
    info: CrateInformation,
}

#[derive(Deserialize, Debug)]
struct CrateInformation {
    max_version: String,
}

pub fn background_check_for_updates() -> mpsc::Receiver<Version> {
    let (sender, receiver) = mpsc::channel();

    let _detached_thread = thread::spawn(move || match check_wrangler_versions() {
        Ok(wrangler_versions) => {
            if wrangler_versions.current != wrangler_versions.latest {
                let _ = sender.send(wrangler_versions.latest);
            }
        }
        Err(e) => log::debug!("could not determine if update is needed:\n{}", e),
    });

    receiver
}
