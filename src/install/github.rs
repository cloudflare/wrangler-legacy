use crate::install::krate::Krate;
use crate::install::target;

// DOWNLOADS: tuples of (tool_name, owner) we know to download from GitHub.
pub const DOWNLOADS: &[(&str, &str)] = &[
    ("cargo-generate", "ashleygwilliams"),
    ("wasm-pack", "rustwasm"),
];

pub fn prebuilt_url(tool_name: &str, tool_owner: &str, version: &str) -> Option<String> {
    if tool_name == "wranglerjs" {
        Some(format!(
            "https://workers.cloudflare.com/get-wranglerjs-binary/{0}/v{1}.tar.gz",
            tool_name, version
        ))
    } else {
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
            "https://workers.cloudflare.com/get-binary/{0}/{1}/v{2}/{3}.tar.gz",
            tool_owner, tool_name, version, target
        );
        Some(url)
    }
}

pub fn get_latest_version(tool_name: &str) -> Result<String, failure::Error> {
    Ok(Krate::new(tool_name)?.max_version)
}
