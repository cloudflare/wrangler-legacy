mod krate;
mod target;

use failure::ResultExt;
use krate::Krate;

use binary_install::{Cache, Download};
use which::which;

pub fn install(tool_name: &str, owner: &str, cache: &Cache) -> Result<Download, failure::Error> {
    if let Ok(path) = which(tool_name) {
        log::debug!("found global {} binary at: {}", tool_name, path.display());
        return Ok(Download::at(path.parent().unwrap()));
    }

    let latest_version = get_latest_version(tool_name)?;
    let download = download_prebuilt(cache, tool_name, owner, &latest_version);
    Ok(download.context(format!("could not download pre-built `{}`.", tool_name))?)
}

fn download_prebuilt(
    cache: &Cache,
    tool_name: &str,
    owner: &str,
    version: &str,
) -> Result<Download, failure::Error> {
    println!("⬇️ Installing {}...", tool_name);
    let url = match prebuilt_url(tool_name, owner, version) {
        Some(url) => url,
        None => failure::bail!(format!(
            "no prebuilt {} binaries are available for this platform",
            tool_name
        )),
    };

    let binary_name = if cfg!(target_os = "windows") {
        format!("{}.exe", tool_name)
    } else {
        tool_name.to_owned()
    };
    let binaries = &[binary_name.as_str()];

    match cache.download(true, tool_name, binaries, &url)? {
        Some(download) => Ok(download),
        None => failure::bail!("{} is not installed!", tool_name),
    }
}

fn prebuilt_url(tool_name: &str, owner: &str, version: &str) -> Option<String> {
    let target = if target::LINUX && target::x86_64 {
        "x86_64-unknown-linux-musl"
    } else if target::MACOS && target::x86_64 {
        "x86_64-apple-darwin"
    } else if target::WINDOWS && target::x86_64 {
        "x86_64-pc-windows-msvc"
    } else {
        return None;
    };

    let url = format!(
        "https://github.com/{0}/{1}/releases/download/v{2}/{1}-v{2}-{3}.tar.gz",
        owner, tool_name, version, target
    );
    log::trace!("URL to download {}", &url);
    Some(url)
}

fn get_latest_version(tool_name: &str) -> Result<String, failure::Error> {
    Ok(Krate::new(tool_name)?.max_version)
}
