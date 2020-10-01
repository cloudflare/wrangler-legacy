use std::{
    ffi::OsStr,
    fmt::Debug,
    fs,
    path::{Path, PathBuf},
};

use bytesize::ByteSize;

mod js;
mod wasm;

use js::check_js;
use wasm::check_wasm;

const JS_FILE_NAME: &str = "worker.js";
const JS_SOURCEMAP_FILE_NAME: &str = "worker.js.map";
const WASM_FILE_NAME: &str = "wasm.wasm"; // ? seems silly lol "wasm.wasm" but ok

enum FileType {
    JavaScript(PathBuf),
    WebAssembly(Option<PathBuf>),
    JavaScriptSourceMap(Option<PathBuf>),
}

impl FileType {
    pub fn check(&self) -> Result<Option<String>, failure::Error> {
        Ok(match self {
            FileType::JavaScript(path) => Some(check_js(path)?),
            FileType::WebAssembly(Some(path)) => Some(check_wasm(path)?),
            FileType::JavaScriptSourceMap(Some(path)) => Some(check_file_size(path)?),
            _ => None,
        })
    }
}

pub struct BundlerOutput(Vec<FileType>);

impl BundlerOutput {
    // it would be cool if this was TryFrom but we need them to
    // stabilize generic associated traits <3
    pub fn new<P: AsRef<Path> + Debug>(output_dir: P) -> Result<Self, failure::Error> {
        if let Some(js_file) = find_and_normalize(&output_dir, JS_FILE_NAME)? {
            let wasm_file = find_and_normalize(&output_dir, JS_SOURCEMAP_FILE_NAME)?;
            let sourcemap_file = find_and_normalize(&output_dir, WASM_FILE_NAME)?;

            Ok(Self(vec![
                FileType::JavaScript(js_file),
                FileType::WebAssembly(wasm_file),
                FileType::JavaScriptSourceMap(sourcemap_file),
            ]))
        } else {
            Err(failure::format_err!(
                "There doesn't appear to be any javascript in {:?}",
                output_dir
            ))
        }
    }

    pub fn check(&self) -> Result<String, failure::Error> {
        // this looks really intimidating but i promise it's not! all we're doing is
        // 1. creating an iterator over the files
        // 2. executing each file's "check" impl
        // 3. Running an accumulator on the results
        // 3a. if the check failed, short-circuiting with the error
        // 3b. if the check succeeded, checking if there was a message
        // 3c. if there is a message, appending it to the output
        // 4. Storing the combined check messages into the "output" string
        // TODO use rayon::par_iter instead of iter if it takes too long
        // TODO use the nice formatting instead of \n
        let output = self.0.iter().map(|file| file.check()).try_fold(
            "".to_string(),
            |output, result| -> Result<String, failure::Error> {
                if let Some(message) = result? {
                    Ok(format!("{}\n{}", output, message))
                } else {
                    Ok(output)
                }
            },
        )?;

        Ok(output)
    }
}

fn find_and_normalize<P, S>(dir: P, file_name: S) -> Result<Option<PathBuf>, failure::Error>
where
    P: AsRef<Path> + Debug,
    S: Into<String> + Debug,
{
    let name: String = file_name.into();

    // This just panics if it cant find a period. If only we could
    // coerce NoneErrors into our error handling.......anyhow.......
    let extension = if let Some(slice) = name.rsplit('.').take(1).next() {
        OsStr::new(slice)
    } else {
        return Err(failure::format_err!("Failed to find a period in {}", name));
    };
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

/// I didn't feel like this deserved its own `common.rs`
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

    #[cfg(test)]
    mod file_with_extension {
        use super::super::file_with_extension;
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
        fn it_errors_with_multiple_files_of_same_extension_are_present(
        ) -> Result<(), failure::Error> {
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
        fn it_returns_none_when_no_files_with_the_extension_are_present(
        ) -> Result<(), failure::Error> {
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

    #[cfg(test)]
    mod normalize_filename {
        use super::super::normalize_filename;
        use std::ffi::OsStr;

        #[test]
        fn it_renames_files_when_necessary() -> Result<(), failure::Error> {
            let file = tempfile::NamedTempFile::new()?;
            let file_path = file.path().to_path_buf();

            let canonical_name = OsStr::new("nice_file_name.example");

            let final_file = normalize_filename(file_path, canonical_name)?;

            assert_eq!(final_file.file_name().unwrap(), canonical_name);

            Ok(())
        }

        #[test]
        fn it_does_nothing_to_correctly_named_files() -> Result<(), failure::Error> {
            let file = tempfile::NamedTempFile::new()?;
            let file_path = file.path().to_path_buf();

            let canonical_name = file_path.file_name().unwrap();

            assert_eq!(file_path.file_name().unwrap(), canonical_name);

            let final_file = normalize_filename(file_path.clone(), canonical_name)?;

            assert_eq!(final_file.file_name().unwrap(), canonical_name);

            Ok(())
        }
    }

    #[cfg(test)]
    mod check_file_size {
        use super::super::check_file_size;
        use bytesize::MB;
        use rand::{distributions::Alphanumeric, thread_rng, Rng};
        use std::{convert::TryInto, io::Write};

        #[test]
        fn its_ok_with_small_files() -> Result<(), failure::Error> {
            let file = tempfile::NamedTempFile::new()?;
            let path_buf = file.path().to_path_buf();

            assert!(check_file_size(&path_buf).is_ok());

            Ok(())
        }

        #[test]
        fn it_errors_when_file_is_too_big() -> Result<(), failure::Error> {
            let mut file = tempfile::NamedTempFile::new()?;

            let data = thread_rng()
                .sample_iter(&Alphanumeric)
                .take((2 * MB).try_into().unwrap())
                .collect::<String>();

            writeln!(file, "{}", data)?;

            let path_buf = file.path().to_path_buf();

            assert!(check_file_size(&path_buf).is_err());

            Ok(())
        }
    }
}
