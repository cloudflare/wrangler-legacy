use std::env;
use std::path::{Path, PathBuf};

pub const DEFAULT_CONFIG_FILE_NAME: &str = "default.toml";

pub fn get_wrangler_home_dir() -> PathBuf {
    if let Ok(value) = env::var("WRANGLER_HOME") {
        log::info!("Using $WRANGLER_HOME: {}", value);
        Path::new(&value).to_path_buf()
    } else {
        log::info!("No $WRANGLER_HOME detected, using $HOME");
        dirs::home_dir()
            .expect("Could not find home directory")
            .join(".wrangler")
    }
}

pub fn get_global_config_path() -> PathBuf {
    let home_dir = get_wrangler_home_dir();
    let global_config_file = home_dir.join("config").join(DEFAULT_CONFIG_FILE_NAME);
    log::info!("Using global config file: {}", global_config_file.display());
    global_config_file
}
