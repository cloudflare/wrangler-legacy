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

pub fn global_config(token: bool) -> Result<(), failure::Error> {
    let mut user = GlobalUser {
        email: None,
        api_key: None,
        api_token: None,
    };

    if token {
        println!("Enter API token: ");
        let mut api_token_str: String = read!("{}\n");
        api_token_str.truncate(api_token_str.trim_end().len());
        if !api_token_str.is_empty() {
            user.api_token = Some(api_token_str);
        }
    } else {
        println!("Enter email: ");
        let mut email_str: String = read!("{}\n");
        email_str.truncate(email_str.trim_end().len());
        if !email_str.is_empty() {
            user.email = Some(email_str);
        }

        println!("Enter API key: ");
        let mut api_key_str: String = read!("{}\n");
        api_key_str.truncate(api_key_str.trim_end().len());
        if !api_key_str.is_empty() {
            user.api_key = Some(api_key_str);
        }
    }

    let toml = toml::to_string(&user)?;

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
