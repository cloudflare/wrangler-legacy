use std::env;
use std::path::{Path, PathBuf};

use anyhow::Result;

pub const DEFAULT_CONFIG_FILE_NAME: &str = "default.toml";

pub fn get_wrangler_home_dir() -> Result<PathBuf> {
    let config_dir = if let Ok(value) = env::var("WRANGLER_HOME") {
        log::info!("Using $WRANGLER_HOME: {}", value);
        Path::new(&value).to_path_buf()
    } else {
        log::info!("No $WRANGLER_HOME detected, using $HOME");
        dirs::home_dir()
            .expect("Could not find home directory")
            .join(".wrangler")
    };
    Ok(config_dir)
}

pub fn get_global_config_path() -> Result<PathBuf> {
    let home_dir = get_wrangler_home_dir()?;
    let global_config_file = home_dir.join("config").join(DEFAULT_CONFIG_FILE_NAME);
    log::info!("Using global config file: {}", global_config_file.display());
    Ok(global_config_file)
}
