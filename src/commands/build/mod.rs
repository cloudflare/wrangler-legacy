pub mod wranglerjs;

use crate::settings::project::Project;
use crate::workers;

pub fn build(project: &Project) -> Result<(), failure::Error> {
    workers::build(project)?;
    Ok(())
}
