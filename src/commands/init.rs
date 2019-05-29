use crate::emoji;
use crate::settings::project::{Project, ProjectType};
use std::path::Path;

pub fn init(name: Option<&str>, project_type: Option<ProjectType>) -> Result<(), failure::Error> {
    if Path::new("./wrangler.toml").exists() {
        failure::bail!("A wrangler.toml file already exists! Please remove it before running this command again.");
    }
    let dirname = get_current_dirname()?;
    let name = name.unwrap_or_else(|| &dirname);
    let project_type = project_type.unwrap_or_default();
    Project::generate(name.to_string(), project_type, true)?;
    println!("{} Succesfully created a `wrangler.toml`", emoji::SPARKLES);
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
