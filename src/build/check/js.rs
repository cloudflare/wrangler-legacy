use std::{
    ffi::OsString,
    fmt::Debug,
    fs,
    path::{Path, PathBuf},
};

use bytesize::ByteSize;

const WORKER_FILE_NAME: &str = "worker.js";

pub fn check_js<P: AsRef<Path> + Debug>(file_path: P) -> Result<String, failure::Error> {
    let worker_js = normalize_filename(file_path)?;
    let file_size = ByteSize::b(worker_js.metadata()?.len());

    Ok(format!("worker.js OK! Final size: {}", file_size))
}

fn normalize_filename<P: AsRef<Path> + Debug>(file_path: P) -> Result<PathBuf, failure::Error> {
    match file_path.as_ref().file_name() {
        None => Err(failure::format_err!("{:?} does not exist!", file_path)),
        Some(name) if name == OsString::from(WORKER_FILE_NAME) => {
            Ok(PathBuf::from(file_path.as_ref()))
        }
        Some(name) => {
            println!("Renaming {:?} to {:?}", name, WORKER_FILE_NAME);
            let worker_js = file_path.as_ref().parent().unwrap().join(WORKER_FILE_NAME);
            fs::rename(&file_path, &worker_js)?;
            Ok(worker_js)
        }
    }
}
