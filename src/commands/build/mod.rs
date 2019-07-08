pub mod wranglerjs;

use crate::settings::project::Project;

pub fn build(project: &Project) -> Result<(), failure::Error> {
    project.build()
}
