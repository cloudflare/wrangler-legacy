use std::path::{Path, PathBuf};

use crate::commands;
use crate::commands::validate_worker_name;
use crate::settings::target::{Manifest, Site, TargetType};
use crate::terminal::message;

pub fn init(
    name: Option<&str>,
    target_type: Option<TargetType>,
    site: bool,
) -> Result<(), failure::Error> {
    if Path::new("./wrangler.toml").exists() {
        failure::bail!("A wrangler.toml file already exists! Please remove it before running this command again.");
    }
    let dirname = get_current_dirname()?;
    let name = name.unwrap_or_else(|| &dirname);
    validate_worker_name(name)?;

    let target_type = target_type.unwrap_or_default();
    let config_path = PathBuf::from("./");
    let initialized_site = if site { Some(Site::default()) } else { None };
    let manifest = Manifest::generate(
        name.to_string(),
        Some(target_type),
        &config_path,
        initialized_site,
    )?;
    message::success("Succesfully created a `wrangler.toml`");

    if site {
        let env = None;
        let target = manifest.get_target(env)?;
        commands::build::wranglerjs::scaffold_site_worker(&target)?;
    }
    Ok(())
}

fn get_current_dirname() -> Result<String, failure::Error> {
    let current_path = std::env::current_dir()?;
    let parent = current_path.parent();
    let dirname = match parent {
        Some(parent) => current_path.strip_prefix(parent)?.display().to_string(),
        None => "worker".to_string(),
    };
    Ok(dirname)
}
