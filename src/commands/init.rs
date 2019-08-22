use crate::settings::project::{Project, ProjectType};
use crate::terminal::message;
use std::path::Path;

pub fn init(
    name: Option<&str>,
    project_type: Option<ProjectType>,
    project_dir: &Path,
) -> Result<(), failure::Error> {
    if project_dir.join("wrangler.toml").exists() {
        failure::bail!("A wrangler.toml file already exists! Please remove it before running this command again.");
    }
    let dirname = get_current_dirname()?;
    let name = name.unwrap_or_else(|| &dirname);
    let project_type = project_type.unwrap_or_default();
    Project::generate(name.to_string(), project_type, project_dir, true)?;
    message::success("Succesfully created a `wrangler.toml`");
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
