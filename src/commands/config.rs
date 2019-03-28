use std::fs;
use std::path::Path;

use crate::user::settings::Settings;

pub fn config(email: &str, api_key: &str) -> Result<(), failure::Error> {
    let s = Settings {
        email: email.to_string(),
        api_key: api_key.to_string(),
    };

    let toml = toml::to_string(&s)?;

    let config_dir =
        Path::new(&dirs::home_dir().expect("🚧 Could not determine home directory. 🚧"))
            .join(".wrangler")
            .join("config");
    fs::create_dir_all(&config_dir)?;

    let config_file = config_dir.join("default.toml");
    fs::write(&config_file, &toml)?;

    println!(
        "✨ Successfully configured. You can find your configuration file at: {}. ✨",
        &config_file.to_string_lossy()
    );
    Ok(())
}
