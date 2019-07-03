use std::path::Path;

// FIXME: move package in javascript
use crate::commands::publish::package::Package;
use crate::terminal::message;
use crate::worker_bundle::WorkerBundle;

pub fn run_build() -> Result<WorkerBundle, failure::Error> {
    message::info("JavaScript project found. Skipping unnecessary build!");
    let package = Package::new("./")?;
    Ok(WorkerBundle {
        script_path: Path::new(&package.main()?).to_path_buf(),
        bindings: vec![],

        // let the WorkerBundle generate the metadata file based on the bindings
        metadata_path: None,
        out_root: None,
    })
}
