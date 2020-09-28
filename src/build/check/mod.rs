use std::{ffi::OsStr, fs};
use std::{
    fmt::Debug,
    path::{Path, PathBuf},
};

mod js;
mod wasm;

use js::check_js;
use wasm::check_wasm;

// TODO: i'm not sure whether it's better to do a pointer to an array of &str, or specify the length.
// my gut feeling is that specifying the length is better since the lengths are known at compile time
const UNAVAILABLE_BUILTINS: [&str; 2] = ["eval", "new Function"];
const AVAILABLE_BUILTINS: [&str; 5] = ["atob", "btoa", "TextEncoder", "TextDecoder", "URL"];
const AVAILABLE_WITHIN_REQUEST_CONTEXT: [&str; 5] = [
    "setInterval",
    "clearInterval",
    "setTimeout",
    "clearTimeout",
    "fetch",
];

/// Run some sanity checks on a given output directory before
/// uploading. Namely:
///
/// 1. Is there a single JS file named worker.js?
/// 2. Is worker.js small enough?
/// 3. Is worker.js using only explicitly allowed features?
/// 4. Is there optionally a single WASM file named `TODO: wasm file name`?
/// 5. If so, is the file being imported correctly?
///
/// Most of these can be fixed, but some (multiple JS files, file too big, and banned functionality)
/// can't. Encountering anything unfixable will result in an error.
/// If everything goes smoothly, an Ok(String) will be returned with some info
/// about the check process.
///
pub fn check_output_dir<P: AsRef<Path> + Debug>(dir: P) -> Result<String, failure::Error> {
    let wasm_file = file_with_extension(&dir, OsStr::new("wasm"))?;
    let js_file = file_with_extension(&dir, OsStr::new("js"))?;

    let js_result = if let Some(path) = js_file {
        check_js(path)?
    } else {
        return Err(failure::format_err!(
            "Failed to find any JS files in ${:?}",
            dir,
        ));
    };

    let wasm_result = if let Some(path) = wasm_file {
        check_wasm(path)?
    } else {
        // TODO this should be like an info-level warning
        format!("No .wasm files found in {:?}", dir)
    };

    Ok(format!("{}\n{}", js_result, wasm_result))
}

/// Returns either Ok(Some(PathBuf)) if one file with the given extension was found
/// in the directory, or Ok(None) if there weren't any. If multiple files are found
/// with the given extension, returns failure::Error
fn file_with_extension<P: AsRef<Path> + Debug>(
    dir: P,
    extension: &OsStr,
) -> Result<Option<PathBuf>, failure::Error> {
    let mut path: Option<PathBuf> = None;

    for entry in fs::read_dir(&dir)? {
        let entry = entry?;

        // yo dawg, i heard you like conditionals
        // so we put some conditionals in your conditionals so you can `if` while you `if`
        if let Some(ext) = entry.path().extension() {
            if ext == extension {
                match path {
                    Some(_) => {
                        return Err(failure::format_err!(
                            "Found multiple files with extension {:?} in {:?}!",
                            extension,
                            &dir
                        ));
                    }
                    None => path = Some(entry.path()),
                };
            }
        }
    }

    Ok(path)
}
