mod javascript;
mod rust;
pub mod wranglerjs;

use crate::settings::project::{Project, ProjectType};
use crate::worker_bundle::WorkerBundle;

pub fn build(project: &Project) -> Result<WorkerBundle, failure::Error> {
    let worker_bundle = match &project.project_type {
        ProjectType::JavaScript => javascript::run_build()?,
        ProjectType::Rust => rust::run_build()?,
        ProjectType::Webpack => wranglerjs::run_build(project)?,
    };

    worker_bundle
        .persist()
        .expect("could not create persist WorkerBundle");
    worker_bundle.print_stats();

    Ok(worker_bundle)
}
