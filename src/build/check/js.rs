use std::{
    ffi::OsString,
    fmt::Debug,
    fs,
    path::{Path, PathBuf},
};

use bytesize::ByteSize;

const WORKER_FILE_NAME: &str = "worker.js";

// // TODO: i'm not sure whether it's better to do a pointer to an array of &str, or specify the length.
// // my gut feeling is that specifying the length is better since the lengths are known at compile time
// const UNAVAILABLE_BUILTINS: [&str; 2] = ["eval", "new Function"];
// const AVAILABLE_BUILTINS: [&str; 5] = ["atob", "btoa", "TextEncoder", "TextDecoder", "URL"];
// const AVAILABLE_WITHIN_REQUEST_CONTEXT: [&str; 5] = [
//     "setInterval",
//     "clearInterval",
//     "setTimeout",
//     "clearTimeout",
//     "fetch",
// ];

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
