mod krate;
mod target;

use krate::Krate;

use binary_install::{Cache, Download};
use which::which;

pub fn install(cache: &Cache) -> Result<Download, failure::Error> {
    let tool_name = "cargo-generate";
    let owner = "ashleygwilliams";

    if let Ok(path) = which(tool_name) {
        log::debug!("found global {} binary at: {}", tool_name, path.display());
        return Ok(Download::at(path.parent().unwrap()));
    }

    let msg = format!("⬇️ Installing {}...", tool_name);
    println!("{}", msg);

    let latest_version = get_latest_version(tool_name)?;
    let download = download_prebuilt(cache, tool_name, owner, &latest_version);
    match download {
        Ok(download) => Ok(download),
        Err(_) => {
            failure::bail!("could not download pre-built `{}`.", tool_name);
        }
    }
}

fn download_prebuilt(
    cache: &Cache,
    tool_name: &str,
    owner: &str,
    version: &str,
) -> Result<Download, failure::Error> {
    let url = match prebuilt_url(tool_name, owner, version) {
        Some(url) => url,
        None => failure::bail!(format!(
            "no prebuilt {} binaries are available for this platform",
            tool_name
        )),
    };
    println!("{}", url);
    let binaries = &[tool_name];
    match cache.download(true, tool_name, binaries, &url)? {
        Some(download) => {
            println!("success with {:?}", download);
            Ok(download)
        }
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

    Some(format!(
        "https://github.com/{0}/{1}/releases/download/v{2}/cargo-generate-v{2}-{3}.tar.gz",
        owner, tool_name, version, target
    ))
}

fn get_latest_version(tool_name: &str) -> Result<String, failure::Error> {
    Ok(Krate::new(tool_name)?.max_version)
}
