use std::{
    ffi::OsStr,
    fmt::Debug,
    fs,
    path::{Path, PathBuf},
};

use bytesize::ByteSize;
use wasmparser::WasmFeatures;

mod config;
mod js;
mod wasm;

use self::config::MAX_FILE_SIZE;
use js::{JavaScript, JavaScriptLinterArgs};
use wasm::WebAssembly;

const JS_FILE_NAME: &str = "worker.js";
const JS_SOURCEMAP_FILE_NAME: &str = "worker.js.map";
const WASM_FILE_NAME: &str = "wasm.wasm"; // ? seems silly lol "wasm.wasm" but ok
const WASM_TEXT_FILE_NAME: &str = "wasm.wat"; // in the spec
const WASM_TEXT_EXTENDED_FILE_NAME: &str = "wasm.wast"; // not in the spec, but commonly used and easier to read.

// this would simplify a lot of this code
// https://github.com/rust-lang/rust/issues/63063
// type PathLike = impl AsRef<Path> + Debug;

/// If a struct is Parseable, that means that it is able to parse some
/// some given input into Self, with the potential for failure.
/// ```
/// # trait Parseable<ParserInput>: Sized {
/// #    fn parse(input: &ParserInput) -> Result<Self, failure::Error>;
/// # }
/// struct SmallNumber(u8);
/// impl Parseable<u8> for SmallNumber {
///     fn parse(input: &u8) -> Result<Self, failure::Error> {
///         Ok(Self(*input))
///     }
/// }
///
/// let n = SmallNumber::parse(&8);
///
/// assert!(n.is_ok());
/// ```
pub trait Parseable<ParserInput>: Sized {
    fn parse(input: &ParserInput) -> Result<Self, failure::Error>;
}

/// If a struct is Lintable, that means that when given some parameters,
/// it's able to check itself according to those criteria and fail if
/// any are not met.
/// ```
/// # trait Lintable<ArgsType> {
/// #     fn lint(&self, args: ArgsType) -> Result<(), failure::Error>;
/// # }
/// struct SmallNumber(u8);
/// impl Lintable<u8> for SmallNumber {
///     fn lint(&self, max_size: u8) -> Result<(), failure::Error> {
///         if self.0 > max_size {
///             Err(failure::err_msg("Number is too big!"))
///         } else {
///             Ok(())
///         }
///     }
/// }
///
/// let n = SmallNumber(3);
///
/// assert!(n.lint(6).is_ok());
/// assert!(n.lint(1).is_err());
/// ```
pub trait Lintable<ArgsType> {
    fn lint(&self, args: ArgsType) -> Result<(), failure::Error>;
}

/// If a struct can Validate, then it must also be Parseable and Lintable.
/// Typically, this acts as a wrapper for self.lint(), and allows any
/// top level args to lint() to be hidden from whoever is calling .validate(),
/// which accepts no arguments.
/// ```
/// # trait Parseable<ParserInput>: Sized {
/// #    fn parse(input: &ParserInput) -> Result<Self, failure::Error>;
/// # }
/// # trait Lintable<ArgsType> {
/// #     fn lint(&self, args: ArgsType) -> Result<(), failure::Error>;
/// # }
/// # pub trait Validate<LinterArgsType, ParserInput>:
/// #     Lintable<LinterArgsType> + Parseable<ParserInput>
/// # {
/// #     fn validate(&self) -> Result<(), failure::Error>;
/// # }
/// struct SmallNumber(u8);
/// const TOO_BIG: u8 = 5;
///
/// impl Parseable<u8> for SmallNumber {
///     fn parse(input: &u8) -> Result<Self, failure::Error> {
///         Ok(Self(*input))
///     }
/// }
///
/// impl Lintable<u8> for SmallNumber {
///     fn lint(&self, max_size: u8) -> Result<(), failure::Error> {
///         if self.0 > max_size {
///             Err(failure::err_msg("Number is too big!"))
///         } else {
///             Ok(())
///         }
///     }
/// }
///
/// impl Validate<u8, u8> for SmallNumber {
///     fn validate(&self) -> Result<(), failure::Error> {
///         self.lint(TOO_BIG)
///     }
/// }
///
/// let small = SmallNumber::parse(&4).unwrap();
/// let big = SmallNumber::parse(&6).unwrap();
///
/// assert!(small.validate().is_ok());
/// assert!(big.validate().is_err());
/// ```
pub trait Validate<LinterArgsType, ParserInput>:
    Lintable<LinterArgsType> + Parseable<ParserInput>
{
    fn validate(&self) -> Result<(), failure::Error>;
}

/// Represents the output of any tool capable of (com|trans)piling source code
/// to javascript and webassembly. See [JavaScript] and [WebAssembly] for more details
/// on their respective implementations.
pub struct BundlerOutput {
    javascript: JavaScript,
    webassembly: Option<WebAssembly>,
}

impl<P> Parseable<P> for BundlerOutput
where
    P: AsRef<Path> + Debug,
{
    fn parse(input: &P) -> Result<Self, failure::Error> {
        // there needs to be javascript, so err right away if we don't find any
        let js_file = match find_and_normalize(input, JS_FILE_NAME)? {
            Some(path) => path,
            None => {
                return Err(failure::format_err!(
                    "There doesn't appear to be any javascript in {:?}",
                    input
                ))
            }
        };
        // we want to fail as soon as possible if anything is too big
        check_file_size(&js_file, MAX_FILE_SIZE)?;

        // source map is optional
        let source_map_file = find_and_normalize(input, JS_SOURCEMAP_FILE_NAME)?;

        if let Some(file) = &source_map_file {
            check_file_size(file, MAX_FILE_SIZE)?
        };

        let javascript = JavaScript::parse(&(js_file, source_map_file))?;

        let webassembly = if let Some(wasm_file) = find_and_normalize(input, WASM_FILE_NAME)? {
            check_file_size(&wasm_file, MAX_FILE_SIZE)?;
            // we only need to parse one of these, even if they're both present. i guess we should prefer WAST over WAT.
            let text_file =
                if let Some(wast) = find_and_normalize(input, WASM_TEXT_EXTENDED_FILE_NAME)? {
                    Some(wast)
                } else if let Some(wat) = find_and_normalize(input, WASM_TEXT_FILE_NAME)? {
                    Some(wat)
                } else {
                    None
                };

            Some(WebAssembly::parse(&(wasm_file, text_file))?)
        } else {
            None
        };

        Ok(Self {
            javascript,
            webassembly,
        })
    }
}

type BundlerOutputLinterArgs = (JavaScriptLinterArgs, WasmFeatures);
impl Lintable<BundlerOutputLinterArgs> for BundlerOutput {
    fn lint(&self, (js_args, wasm_args): BundlerOutputLinterArgs) -> Result<(), failure::Error> {
        self.javascript.lint(js_args)?;
        if let Some(webassembly) = &self.webassembly {
            webassembly.lint(wasm_args)
        } else {
            Ok(())
        }
    }
}

// i hate to end the AsRef<Path> generic-ness here, but i really
// and truly cannot for the life of me how to specify
// the type parameters when calling output.validate()
impl Validate<BundlerOutputLinterArgs, PathBuf> for BundlerOutput {
    fn validate(&self) -> Result<(), failure::Error> {
        self.javascript.validate()?;
        if let Some(wasm) = &self.webassembly {
            wasm.validate()?
        };

        Ok(())
    }
}

// helper functions

fn find_and_normalize<P, S>(dir: P, file_name: S) -> Result<Option<PathBuf>, failure::Error>
where
    P: AsRef<Path> + Debug,
    S: Into<String> + Debug,
{
    let name: String = file_name.into();

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

fn check_file_size(file: &PathBuf, max_size: ByteSize) -> Result<(), failure::Error> {
    let file_size = ByteSize::b(file.metadata()?.len());
    if file_size > max_size {
        Err(failure::format_err!(
            "{:?} is {}, which exceeds the {} limit!",
            file.file_name().unwrap_or_else(|| OsStr::new("file")),
            file_size,
            max_size
        ))
    } else {
        Ok(())
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
        use bytesize::{ByteSize, MB};
        use rand::{distributions::Alphanumeric, thread_rng, Rng};
        use std::{convert::TryInto, io::Write};

        #[test]
        fn its_ok_with_small_files() -> Result<(), failure::Error> {
            let file = tempfile::NamedTempFile::new()?;
            let path_buf = file.path().to_path_buf();

            assert!(check_file_size(&path_buf, ByteSize::mb(1)).is_ok());

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

            assert!(check_file_size(&path_buf, ByteSize::mb(1)).is_err());

            Ok(())
        }
    }
}
