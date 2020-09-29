use std::{ffi::OsStr, fs};
use std::{
    fmt::Debug,
    path::{Path, PathBuf},
};

use bytesize::ByteSize;
use tokio::{runtime::Runtime, try_join};

mod js;
mod wasm;

use js::check_js;
use wasm::check_wasm;

const JS_FILE_NAME: &str = "worker.js";
const JS_SOURCEMAP_FILE_NAME: &str = "worker.js.map";
const WASM_FILE_NAME: &str = "wasm.wasm"; // ? seems silly lol "wasm.wasm" but ok

pub struct BundlerOutput {
    js_file: PathBuf,
    wasm_file: Option<PathBuf>,
    sourcemap_file: Option<PathBuf>,
}

impl BundlerOutput {
    pub fn new<P: AsRef<Path> + Debug>(output_dir: P) -> Result<Self, failure::Error> {
        if let Some(js_file) = find_and_normalize(&output_dir, JS_FILE_NAME)? {
            let wasm_file = find_and_normalize(&output_dir, JS_SOURCEMAP_FILE_NAME)?;
            let sourcemap_file = find_and_normalize(&output_dir, WASM_FILE_NAME)?;

            Ok(Self {
                js_file,
                wasm_file,
                sourcemap_file,
            })
        } else {
            Err(failure::format_err!(
                "There doesn't appear to be any javascript in {:?}",
                output_dir
            ))
        }
    }

    pub fn check(&self) -> Result<String, failure::Error> {
        // i felt like this should be async but this implementation is probably terrible
        let mut rt = Runtime::new()?;
        let result: Result<(String, String), failure::Error> = rt.block_on(async {
            // as_ref converts &Option<T> into Option<&T>
            try_join!(
                check_js(&self.js_file, self.sourcemap_file.as_ref()),
                check_wasm(self.wasm_file.as_ref())
            )
        });

        match result {
            Ok((js_result, wasm_result)) => Ok(format!("{}\n{}", js_result, wasm_result)),
            Err(e) => Err(e),
        }
    }
}

fn find_and_normalize<P, S>(dir: P, file_name: S) -> Result<Option<PathBuf>, failure::Error>
where
    P: AsRef<Path> + Debug,
    S: Into<String> + Debug,
{
    // i guess it has to be done at some point
    let name: String = file_name.into();

    // This just panics if it cant find a period. If only we could
    // coerce NoneErrors into our error handling.......anyhow.......
    let extension = OsStr::new(name.rsplit('.').take(1).next().unwrap());
    let canonical_name = OsStr::new(&name);

    // path to the file of the given extension as output by the bundler
    let bundler_output_path = file_with_extension(dir, extension)?;

    // normalized filename to what wrangler expects
    Ok(match bundler_output_path {
        None => None,
        Some(file_path) => Some(normalize_filename(file_path, canonical_name)?),
    })
}

/// Returns either Ok(Some(PathBuf)) if one file with the given extension was found
/// in the directory, or Ok(None) if there weren't any. If multiple files are found
/// with the given extension, returns failure::Error
fn file_with_extension<P: AsRef<Path> + Debug>(
    dir: P,
    extension: &OsStr,
) -> Result<Option<PathBuf>, failure::Error> {
    // path to the file of the given extension as output by the bundler
    let mut bundler_output_path: Option<PathBuf> = None;

    // let's see if we can find a file matching the extension we want
    for entry in fs::read_dir(&dir)? {
        let entry = entry?;

        // ideally, this syntax would look like
        // `if let Some(ext) = entry.path().extension() && ext == extension { ... }`
        // but i'm not sure if that's possible. or like, how even...to do that...
        if let Some(ext) = entry.path().extension() {
            if ext == extension {
                match bundler_output_path {
                    Some(_) => {
                        return Err(failure::format_err!(
                            "Found multiple files with extension {:?} in {:?}!",
                            extension,
                            &dir
                        ));
                    }
                    None => bundler_output_path = Some(entry.path()),
                };
            }
        }
    }

    Ok(bundler_output_path)
}

fn normalize_filename(
    file_path: PathBuf,
    canonical_name: &OsStr,
) -> Result<PathBuf, failure::Error> {
    match file_path.file_name() {
        // why have u done this
        None => Err(failure::format_err!(
            "{:?} is not a valid path because it ends in \"..\"",
            file_path
        )),

        // oh hey nice!
        Some(name) if name == canonical_name => Ok(file_path),

        // cmon now let's...let's be nice to wrangler and save us all some writes to disk
        Some(name) => {
            // TODO this warning message should be better and not like...a println
            println!("Renaming {:?} to {:?}", name, canonical_name);
            // i hate unwraps...maybe this could or_else to a tempdir()?
            let canon_file = file_path.parent().unwrap().join(canonical_name);
            fs::rename(&file_path, &canon_file)?;
            Ok(canon_file)
        }
    }
}

/// I didn't feel like this deserved its own `common.rs` but uh...i guess
/// the visibility is technically an issue?
fn check_file_size(file: &PathBuf) -> Result<String, failure::Error> {
    let size = ByteSize::b(file.metadata()?.len());
    if size > ByteSize::mb(1) {
        Err(failure::format_err!(
            "{:?} is {}, which exceeds the 1 MB limit!",
            // i hate unwrap so much but we can't coerce NoneError into failure::Error...anyhow...
            file.file_name().unwrap(),
            size
        ))
    } else {
        Ok(size.to_string())
    }
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
