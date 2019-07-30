use crate::terminal::message;
use std::fs;
use std::fs::File;
#[cfg(not(target_os = "windows"))]
use std::os::unix::fs::PermissionsExt;
use std::path::PathBuf;

use crate::settings::global_user::{get_global_config_dir, GlobalUser};

// set the permissions on the dir, we want to avoid that other user reads to
// file
#[cfg(not(target_os = "windows"))]
pub fn set_file_mode(file: &PathBuf) {
    File::open(&file)
        .unwrap()
        .set_permissions(PermissionsExt::from_mode(0o600))
        .expect("could not set permissions on file");
}

pub fn global_config(email: &str, api_key: &str) -> Result<(), failure::Error> {
    let s = GlobalUser {
        email: email.to_string(),
        api_key: api_key.to_string(),
    };

    let toml = toml::to_string(&s)?;

    let config_dir = get_global_config_dir().expect("could not find global config directory");
    fs::create_dir_all(&config_dir)?;

    let config_file = config_dir.join("default.toml");
    fs::write(&config_file, &toml)?;

    // set permissions on the file
    #[cfg(not(target_os = "windows"))]
    set_file_mode(&config_file);

    message::success(&format!(
        "Successfully configured. You can find your configuration file at: {}",
        &config_file.to_string_lossy()
    ));

    Ok(())
}
