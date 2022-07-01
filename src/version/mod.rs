use std::fs;
use std::path::Path;
use std::str::FromStr;
use std::time::SystemTime;

use crate::settings::get_wrangler_home_dir;
use crate::terminal::message::{Message, StdOut};
use crate::terminal::styles;

use anyhow::Result;
use reqwest::header::USER_AGENT;
use semver::Version;
use serde::{Deserialize, Serialize};

const ONE_DAY: u64 = 60 * 60 * 24;

pub fn check_for_updates() {
    let major_version_message = String::from("A new major version of wrangler is available!\n");
    let minor_version_message = match check_wrangler_versions() {
        Err(e) => {
            log::debug!("could not determine if update is needed:\n{}", e);
            None
        }
        Ok(versions) => {
            if let Some(diff) = versions.is_outdated() {
                Some(format!(
                    "Additionally, a new {} version is available ({})\n",
                    diff, versions.current
                ))
            } else {
                None
            }
        }
    }
    .unwrap_or_else(|| "".to_string());

    let update_message = format!(
        "You can learn more about updating here:\n{}",
        styles::url("https://developers.cloudflare.com/workers/cli-wrangler/install-update#update",)
    );

    let message = format!(
        "{}{}{}",
        major_version_message, minor_version_message, update_message
    );

    StdOut::billboard(&message);
}

#[derive(Debug, Clone)]
struct WranglerVersion {
    /// currently installed version of wrangler
    pub current: Version,

    /// latest version of wrangler on crates.io
    pub latest: Version,

    /// set to true if wrangler version has been checked within a day
    pub checked: bool,
}

/// _how_ outdated is the currently installed version of wrangler.
/// Major is omitted because there will always be a new major
/// version of wrangler
enum VersionDiff {
    Minor,
    Patch,
}

impl std::fmt::Display for VersionDiff {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                VersionDiff::Minor => "minor",
                VersionDiff::Patch => "patch",
            }
        )
    }
}

impl WranglerVersion {
    pub fn is_outdated(&self) -> Option<VersionDiff> {
        if self.checked {
            None
        } else if self.current.minor < self.latest.minor {
            Some(VersionDiff::Minor)
        } else if self.current.patch < self.latest.patch {
            Some(VersionDiff::Patch)
        } else {
            None
        }
    }
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

fn get_installed_version() -> Result<Version> {
    let version = option_env!("CARGO_PKG_VERSION").unwrap_or_else(|| "unknown");
    let parsed_version = Version::parse(version)?;
    Ok(parsed_version)
}

fn check_wrangler_versions() -> Result<WranglerVersion> {
    let config_dir = get_wrangler_home_dir();
    let version_file = config_dir.join("version.toml");
    let current_time = SystemTime::now();

    let mut checked = false;
    let current = get_installed_version()?;

    let latest = match get_version_disk(&version_file) {
        Some(last_checked_version) => {
            let time_since_last_checked =
                current_time.duration_since(last_checked_version.last_checked)?;

            if time_since_last_checked.as_secs() < ONE_DAY {
                checked = true;
                Version::parse(&last_checked_version.latest_version)?
            } else {
                get_latest_version(&current.to_string(), &version_file, current_time)?
            }
        }
        // If version.toml doesn't exist, fetch latest version
        None => get_latest_version(&current.to_string(), &version_file, current_time)?,
    };

    Ok(WranglerVersion {
        current,
        latest,
        checked,
    })
}

/// Reads version out of version file, is `None` if file does not exist or is corrupted
fn get_version_disk(version_file: &Path) -> Option<LastCheckedVersion> {
    match fs::read_to_string(&version_file) {
        Ok(contents) => match LastCheckedVersion::from_str(&contents) {
            Ok(last_checked_version) => Some(last_checked_version),
            Err(_) => None,
        },
        Err(_) => None,
    }
}

fn get_latest_version(
    installed_version: &str,
    version_file: &Path,
    current_time: SystemTime,
) -> Result<Version> {
    let latest_version = get_latest_version_from_api(installed_version)?;
    let updated_file_contents = toml::to_string(&LastCheckedVersion {
        latest_version: latest_version.to_string(),
        last_checked: current_time,
    })?;
    fs::write(&version_file, updated_file_contents)?;
    Ok(latest_version)
}

fn get_latest_version_from_api(installed_version: &str) -> Result<Version> {
    let url = "https://crates.io/api/v1/crates/wrangler";
    let user_agent = format!(
        "wrangler/{} ({})",
        installed_version,
        env!("CARGO_PKG_REPOSITORY")
    );
    let client = reqwest::blocking::Client::builder().build()?;
    let response = client
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
