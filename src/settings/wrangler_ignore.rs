use std::fs;
use std::path::Path;

// REQUIRED_IGNORE_FILES are gitignore-style globs that must be ignored when uploading files
// to workers KV.
pub const REQUIRED_IGNORE_FILES: &[&str] = &["node_modules"];
pub const WRANGLER_IGNORE: &str = ".wranglerignore";

pub fn create_wrangler_ignore_file(config_path: &Path) -> Result<(), failure::Error> {
    fs::File::create(config_path.join(WRANGLER_IGNORE))?;
    Ok(())
}
