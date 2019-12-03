use crate::install::target;

use reqwest::header::{HeaderMap, HeaderValue, ACCEPT, CONTENT_TYPE};
use reqwest::Client;
use serde::{self, Deserialize, Serialize};

use std::collections::HashMap;

use lazy_static::lazy_static;

lazy_static! {
    // TOOL_TO_ID_MAP maps a toolname to its equnix app_id.
    static ref TOOL_TO_ID_MAP: HashMap<&'static str, &'static str> = [
        ("cloudflared", "app_idCzgxYerVD")
    ].iter().copied().collect();
}

// DOWNLOADS: tuples of (app_id, owner) we know to download from equinox.io.
pub const DOWNLOADS: &[(&str, &str)] = &[
    ("cloudflared", "cloudflare"), // First arg is the tool_id for Equinox lookup (this is used in place of a tool name).
];

#[serde_with::skip_serializing_none]
#[derive(Serialize, Deserialize)]
struct EquinoxVersionRequestParams<'a> {
    app_id: &'a str,
    os: &'a str,
    arch: &'a str,
    target_version: Option<&'a str>,
}

#[derive(Debug, Serialize, Deserialize)]
struct EquinoxVersion {
    download_url: String,
    release: EquinoxRelease,
}

#[derive(Debug, Serialize, Deserialize)]
struct EquinoxRelease {
    version: String,
}

pub fn prebuilt_url(tool_name: &str, version: &str) -> Option<String> {
    let app_id = match TOOL_TO_ID_MAP.get(tool_name) {
        Some(id) => id,
        None => "",
    };
    match check(app_id, Some(version)) {
        // If URL exists, we want to download it as a tarball using the .tgz extension.
        Ok(result) => Some(format!("{}.tgz", result.download_url)),
        Err(_) => None,
    }
}

pub fn get_latest_version(tool_name: &str) -> Result<String, failure::Error> {
    let app_id = match TOOL_TO_ID_MAP.get(tool_name) {
        Some(id) => id,
        None => "",
    };
    let latest = check(app_id, None)?.release.version;
    Ok(latest)
}

// If successful, returns version number and download endpoint.
fn check(app_id: &str, version: Option<&str>) -> Result<EquinoxVersion, failure::Error> {
    let (os, arch): (&str, &str) = if target::LINUX && target::x86_64 {
        ("linux", "amd64")
    } else if target::MACOS && target::x86_64 {
        ("darwin", "amd64")
    } else if target::WINDOWS && target::x86_64 {
        ("windows", "amd64")
    } else {
        failure::bail!("Your platform does not support cloudflared at the moment.")
    };

    let params = EquinoxVersionRequestParams {
        app_id,
        os,
        arch,
        target_version: version,
    };

    let client = equinox_client()?;

    let mut res = client
        .post("https://update.equinox.io/check")
        .json(&params)
        .send()?;

    let res_status = res.status();
    let res_text = res.text()?;

    if !res_status.is_success() {
        failure::bail!(format!("{}:{}", res_status, res_text));
    }

    let version: EquinoxVersion = serde_json::from_str(&res_text)?;
    return Ok(version);
}

fn equinox_client() -> Result<Client, failure::Error> {
    let mut headers = HeaderMap::new();
    headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));
    headers.insert(
        ACCEPT,
        HeaderValue::from_static("application/json; q=1; version=1; charset=utf-8"),
    );

    match Client::builder().default_headers(headers).build() {
        Ok(c) => Ok(c),
        Err(e) => failure::bail!(e),
    }
}
