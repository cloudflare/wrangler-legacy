use std::{ffi::OsStr, fs};
use std::{
    fmt::Debug,
    path::{Path, PathBuf},
};

mod js;
mod wasm;

use js::check_js;
use wasm::check_wasm;

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

    // TODO: depending on what the Cloudflare team says, this could be completely unnecessary
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

#[cfg(test)]
mod tests {
    use super::file_with_extension;
    use std::ffi::OsStr;

    #[test]
    fn it_finds_a_file_with_an_extension() -> Result<(), failure::Error> {
        let dir = &tempfile::tempdir()?;
        let file = tempfile::Builder::new()
            .prefix("test_file")
            .suffix(".example")
            .tempfile_in(dir)?;

        assert!(file_with_extension(dir, file.path().extension().unwrap()).is_ok());

        Ok(())
    }

    #[test]
    fn it_errors_with_multiple_files_of_same_extension_are_present() -> Result<(), failure::Error> {
        let dir = &tempfile::tempdir()?;
        let _file_1 = tempfile::Builder::new()
            .prefix("test_file_one")
            .suffix(".example")
            .tempfile_in(dir)?;
        let _file_2 = tempfile::Builder::new()
            .prefix("test_file_two")
            .suffix(".example")
            .tempfile_in(dir)?;

        assert!(file_with_extension(dir, OsStr::new("example")).is_err());

        Ok(())
    }

    #[test]
    fn it_returns_none_when_no_files_with_the_extension_are_present() -> Result<(), failure::Error>
    {
        let dir = tempfile::tempdir()?;

        assert!(file_with_extension(dir, OsStr::new("example"))?.is_none());

        Ok(())
    }

    #[test]
    fn it_errors_when_path_is_not_dir() -> Result<(), failure::Error> {
        let path = &tempfile::NamedTempFile::new()?.into_temp_path();

        assert!(file_with_extension(path, OsStr::new("test")).is_err());

        Ok(())
    }
}
