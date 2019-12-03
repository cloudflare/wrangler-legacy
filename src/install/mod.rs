mod equinox;
mod github;
mod krate;
pub mod target;

use crate::terminal::emoji;

use binary_install::{Cache, Download};
use log::info;
use which::which;

use std::env;
use std::path::Path;
use std::process::Command;

use lazy_static::lazy_static;

lazy_static! {
    static ref CACHE: Cache = get_wrangler_cache().expect("creating binary dependency cache");
}

enum DependencyEndpoint {
    GitHub,
    Equinox,
}

fn get_latest_version(
    downloader: &DependencyEndpoint,
    tool_name: &str,
) -> Result<String, failure::Error> {
    match downloader {
        DependencyEndpoint::GitHub => github::get_latest_version(tool_name),
        DependencyEndpoint::Equinox => equinox::get_latest_version(tool_name),
    }
}

fn prebuilt_url(
    downloader: &DependencyEndpoint,
    tool_name: &str,
    tool_owner: &str,
    version: &str,
) -> Option<String> {
    match downloader {
        DependencyEndpoint::GitHub => github::prebuilt_url(tool_name, tool_owner, version),
        DependencyEndpoint::Equinox => equinox::prebuilt_url(tool_name, version),
    }
}

pub fn install(tool_name: &str, owner: &str) -> Result<Download, failure::Error> {
    let downloader = get_downloader(tool_name, owner)?;

    if let Some(download) = tool_exists(&downloader, tool_name)? {
        return Ok(download);
    }

    let binaries = &[tool_name];
    let latest_version = get_latest_version(&downloader, tool_name)?;
    let download = download_prebuilt(&downloader, tool_name, owner, &latest_version, binaries);
    match download {
        Ok(download) => Ok(download),
        Err(e) => {
            failure::bail!("could not download pre-built `{}` ({}).", tool_name, e);
        }
    }
}

pub fn install_artifact(
    tool_name: &str,
    owner: &str,
    version: &str,
) -> Result<Download, failure::Error> {
    let downloader = get_downloader(tool_name, owner)?;

    if let Some(download) = tool_exists(&downloader, tool_name)? {
        return Ok(download);
    }

    let download = download_prebuilt(&downloader, tool_name, owner, version, &[]);
    match download {
        Ok(download) => Ok(download),
        Err(e) => {
            failure::bail!("could not download pre-built `{}` ({}).", tool_name, e);
        }
    }
}

fn get_downloader(tool_name: &str, owner: &str) -> Result<DependencyEndpoint, failure::Error> {
    let dependency_host: DependencyEndpoint = if github::DOWNLOADS.contains(&(tool_name, owner)) {
        DependencyEndpoint::GitHub
    } else if equinox::DOWNLOADS.contains(&(tool_name, owner)) {
        DependencyEndpoint::Equinox
    } else {
        failure::bail!(
            "The tool {}/{} is not an expected dependency of wrangler",
            owner,
            tool_name
        );
    };
    Ok(dependency_host)
}

fn tool_exists(
    downloader: &DependencyEndpoint,
    tool_name: &str,
) -> Result<Option<Download>, failure::Error> {
    if let Ok(path) = which(tool_name) {
        let no_parent_msg = format!("{} There is no path parent", emoji::WARN);
        log::debug!("found global {} binary at: {}", tool_name, path.display());
        if !tool_needs_update(downloader, tool_name, &path)? {
            return Ok(Some(Download::at(path.parent().expect(&no_parent_msg))));
        }
    }

    Ok(None)
}

fn tool_needs_update(
    downloader: &DependencyEndpoint,
    tool_name: &str,
    path: &Path,
) -> Result<bool, failure::Error> {
    let no_version_msg = format!("failed to find version for {}", tool_name);

    let tool_version_output = Command::new(path.as_os_str())
        .arg("--version")
        .output()
        .expect(&no_version_msg);

    if !tool_version_output.status.success() {
        let error = String::from_utf8_lossy(&tool_version_output.stderr);
        log::debug!("could not find version for {}\n{}", tool_name, error);
        return Ok(true);
    }

    let installed_tool_version = String::from_utf8_lossy(&tool_version_output.stdout);
    let installed_tool_version = match installed_tool_version.split_whitespace().last() {
        None => return Ok(true),
        Some(v) => v,
    };
    let latest_tool_version = get_latest_version(downloader, tool_name)?;
    if installed_tool_version == latest_tool_version {
        log::debug!(
            "installed {} version {} is up to date",
            tool_name,
            installed_tool_version
        );
        return Ok(false);
    }
    log::info!(
        "installed {} version {} is out of date with latest version {}",
        tool_name,
        installed_tool_version,
        latest_tool_version
    );
    Ok(true)
}

fn download_prebuilt(
    downloader: &DependencyEndpoint,
    tool_name: &str,
    owner: &str,
    version: &str,
    binaries: &[&str],
) -> Result<Download, failure::Error> {
    let url = match prebuilt_url(downloader, tool_name, owner, version) {
        Some(url) => url,
        None => failure::bail!(format!(
            "no prebuilt {} binaries are available for this platform",
            tool_name
        )),
    };

    info!("prebuilt artifact {}", url);

    // no binaries are expected; downloading it as an artifact
    let res = if !binaries.is_empty() {
        CACHE.download(true, tool_name, binaries, &url)?
    } else {
        CACHE.download_artifact(tool_name, &url)?
    };

    match res {
        Some(download) => {
            println!("⬇️ Installing {}...", tool_name);
            Ok(download)
        }
        None => failure::bail!("{} is not installed!", tool_name),
    }
}

fn get_wrangler_cache() -> Result<Cache, failure::Error> {
    if let Ok(path) = env::var("WRANGLER_CACHE") {
        Ok(Cache::at(Path::new(&path)))
    } else {
        Cache::new("wrangler")
    }
}
