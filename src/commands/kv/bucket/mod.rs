extern crate base64;

mod manifest;
mod sync;
mod upload;

use data_encoding::HEXLOWER;
use sha2::{Digest, Sha256};

pub use manifest::AssetManifest;
pub use sync::sync;

use std::ffi::OsString;
use std::fs;
use std::path::Path;

use cloudflare::endpoints::workerskv::write_bulk::KeyValuePair;

use ignore::overrides::{Override, OverrideBuilder};
use ignore::{Walk, WalkBuilder};

use crate::settings::target::Target;
use crate::terminal::message;

pub const KEY_MAX_SIZE: usize = 512;
// Oddly enough, metadata.len() returns a u64, not usize.
pub const VALUE_MAX_SIZE: u64 = 10 * 1024 * 1024;

// Returns the hashed key and value pair for all files in a directory.
pub fn directory_keys_values(
    target: &Target,
    directory: &Path,
    verbose: bool,
) -> Result<(Vec<KeyValuePair>, AssetManifest), failure::Error> {
    let mut upload_vec: Vec<KeyValuePair> = Vec::new();
    let mut asset_manifest: AssetManifest = AssetManifest::new();

    let dir_walker = get_dir_iterator(target, directory)?;

    for entry in dir_walker {
        let entry = entry.unwrap();
        let path = entry.path();
        if path.is_file() {
            if verbose {
                message::working(&format!("Preparing {}", path.display()));
            }

            validate_file_size(&path)?;

            let value = std::fs::read(path)?;

            // Need to base64 encode value
            let b64_value = base64::encode(&value);

            let (url_safe_path, key) =
                generate_path_and_key(path, directory, Some(b64_value.clone()))?;

            validate_key_size(&key)?;

            upload_vec.push(KeyValuePair {
                key: key.clone(),
                value: b64_value,
                expiration: None,
                expiration_ttl: None,
                base64: Some(true),
            });

            asset_manifest.insert(url_safe_path, key);
        }
    }
    Ok((upload_vec, asset_manifest))
}

// Returns only the hashed keys for a directory's files.
fn directory_keys_only(target: &Target, directory: &Path) -> Result<Vec<String>, failure::Error> {
    let mut key_vec: Vec<String> = Vec::new();

    let dir_walker = get_dir_iterator(target, directory)?;

    for entry in dir_walker {
        let entry = entry.unwrap();
        let path = entry.path();
        if path.is_file() {
            let value = std::fs::read(path)?;

            // Need to base64 encode value
            let b64_value = base64::encode(&value);

            let (_, key) = generate_path_and_key(path, directory, Some(b64_value))?;

            validate_key_size(&key)?;

            key_vec.push(key);
        }
    }
    Ok(key_vec)
}

// Ensure that all files in upload directory do not exceed the MAX_VALUE_SIZE (this ensures that
// no partial uploads happen). I don't like this functionality (and the similar key length checking
// logic in validate_key_size()) because it duplicates the size checking the API already does--but
// doing a preemptive check like this (before calling the API) will prevent partial bucket uploads
// from happening.
fn validate_file_size(path: &Path) -> Result<(), failure::Error> {
    let metadata = fs::metadata(path)?;
    let file_len = metadata.len();

    if file_len > VALUE_MAX_SIZE {
        failure::bail!(
            "File `{}` of {} bytes exceeds the maximum value size limit of {} bytes",
            path.display(),
            file_len,
            VALUE_MAX_SIZE
        );
    }
    Ok(())
}

fn validate_key_size(key: &str) -> Result<(), failure::Error> {
    if key.len() > KEY_MAX_SIZE {
        failure::bail!(
            "Path `{}` of {} bytes exceeds the maximum key size limit of {} bytes",
            key,
            key.len(),
            KEY_MAX_SIZE
        );
    }
    Ok(())
}

const REQUIRED_IGNORE_FILES: &[&str] = &["node_modules"];
const NODE_MODULES: &str = "node_modules";

fn get_dir_iterator(target: &Target, directory: &Path) -> Result<Walk, failure::Error> {
    // The directory provided should never be node_modules!
    match directory.file_name() {
        Some(name) => {
            if name == NODE_MODULES {
                failure::bail!("Your directory of files to upload cannot be named node_modules.");
            }
        }
        _ => (),
    }

    let ignore = build_ignore(target, directory)?;
    Ok(WalkBuilder::new(directory).overrides(ignore).build())
}

fn build_ignore(target: &Target, directory: &Path) -> Result<Override, failure::Error> {
    let mut required_override = OverrideBuilder::new(directory);
    // First include files that must be ignored.
    for ignored in REQUIRED_IGNORE_FILES {
        required_override.add(&format!("!{}", ignored))?;
        log::info!("Ignoring {}", ignored);
    }

    if let Some(site) = &target.site {
        // If `include` present, use it and don't touch the `exclude` field
        if let Some(included) = &site.include {
            for i in included {
                required_override.add(&i)?;
                log::info!("Including {}", i);
            }
        // If `exclude` only present, ignore anything in it.
        } else if let Some(excluded) = &site.exclude {
            for e in excluded {
                required_override.add(&format!("!{}", e))?;
                log::info!("Ignoring {}", e);
            }
        }
    }

    let exclude = required_override.build()?;
    Ok(exclude)
}

// Courtesy of Steve Klabnik's PoC :) Used for bulk operations (write, delete)
fn generate_url_safe_path(path: &Path) -> Result<String, failure::Error> {
    // first, we have to re-build the paths: if we're on Windows, we have paths with
    // `\` as separators. But we want to use `/` as separators. Because that's how URLs
    // work.
    let mut path_with_forward_slash = OsString::new();

    for (i, component) in path.components().enumerate() {
        // we don't want a leading `/`, so skip that
        if i > 0 {
            path_with_forward_slash.push("/");
        }

        path_with_forward_slash.push(component);
    }

    // if we have a non-utf8 path here, it will fail, but that's not realistically going to happen
    let path = path_with_forward_slash
        .to_str()
        .unwrap_or_else(|| panic!("found a non-UTF-8 path, {:?}", path_with_forward_slash));

    Ok(path.to_string())
}

// Adds the SHA-256 hash of the path's file contents to the url-safe path of a file to
// generate a versioned key for the file and its contents. Returns the url-safe path prefix
// for the key, as well as the key with hash appended.
// e.g (sitemap.xml, sitemap.ec717eb2131fdd4fff803b851d2aa5b1dc3e0af36bc3c8c40f2095c747e80d1e.xml)
pub fn generate_path_and_key(
    path: &Path,
    directory: &Path,
    value: Option<String>,
) -> Result<(String, String), failure::Error> {
    // strip the bucket directory from both paths for ease of reference.
    let relative_path = path.strip_prefix(directory).unwrap();

    let url_safe_path = generate_url_safe_path(relative_path)?;

    let path_with_hash = if let Some(value) = value {
        let digest = get_digest(value)?;

        generate_path_with_hash(relative_path, digest)?
    } else {
        url_safe_path.to_owned()
    };

    Ok((url_safe_path, path_with_hash))
}

fn get_digest(value: String) -> Result<String, failure::Error> {
    let mut hasher = Sha256::new();
    hasher.input(value);
    let digest = hasher.result();
    let hex_digest = HEXLOWER.encode(digest.as_ref());
    Ok(hex_digest)
}

// Assumes that `path` is a file (called from a match branch for path.is_file())
// Assumes that `hashed_value` is a String, not an Option<String> (called from a match branch for value.is_some())
fn generate_path_with_hash(path: &Path, hashed_value: String) -> Result<String, failure::Error> {
    if let Some(file_stem) = path.file_stem() {
        let mut file_name = file_stem.to_os_string();
        let extension = path.extension();

        file_name.push(".");
        file_name.push(hashed_value);
        if let Some(ext) = extension {
            file_name.push(".");
            file_name.push(ext);
        }

        let new_path = path.with_file_name(file_name);

        Ok(generate_url_safe_path(&new_path)?)
    } else {
        failure::bail!("no file_stem for path {}", path.display())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use regex::Regex;
    use std::fs;
    use std::path::{Path, PathBuf};

    use crate::settings::target::{Site, Target, TargetType};

    fn make_target(site: Site) -> Target {
        Target {
            account_id: "".to_string(),
            kv_namespaces: None,
            name: "".to_string(),
            target_type: TargetType::JavaScript,
            route: None,
            routes: None,
            webpack_config: None,
            zone_id: None,
            site: Some(site),
        }
    }

    #[test]
    fn it_can_ignore_node_modules() {
        let target = make_target(Site {
            bucket: "fake".to_string(),
            entry_point: None,
            include: None,
            exclude: None,
        });

        let test_dir = "test1";
        // If test dir already exists, delete it.
        if fs::metadata(test_dir).is_ok() {
            fs::remove_dir_all(test_dir).unwrap();
        }

        fs::create_dir_all(format!("{}/node_modules", test_dir)).unwrap();
        let test_pathname = format!("{}/node_modules/ignore_me.txt", test_dir);
        let test_path = PathBuf::from(&test_pathname);
        fs::File::create(&test_path).unwrap();

        let files: Vec<_> = get_dir_iterator(&target, Path::new(test_dir))
            .unwrap()
            .map(|entry| entry.unwrap().path().to_owned())
            .collect();

        assert!(!files.contains(&test_path));

        fs::remove_dir_all(test_dir).unwrap();
    }

    #[test]
    fn it_can_ignore_hidden() {
        let target = make_target(Site {
            bucket: "fake".to_string(),
            entry_point: None,
            include: None,
            exclude: None,
        });

        let test_dir = "test2";
        // If test dir already exists, delete it.
        if fs::metadata(test_dir).is_ok() {
            fs::remove_dir_all(test_dir).unwrap();
        }

        fs::create_dir(test_dir).unwrap();
        let test_pathname = format!("{}/.ignore_me.txt", test_dir);
        let test_path = PathBuf::from(&test_pathname);
        fs::File::create(&test_path).unwrap();

        let files: Vec<_> = get_dir_iterator(&target, Path::new(test_dir))
            .unwrap()
            .map(|entry| entry.unwrap().path().to_owned())
            .collect();

        assert!(!files.contains(&test_path));

        fs::remove_dir_all(test_dir).unwrap();
    }

    #[test]
    fn it_can_allow_unfiltered_files() {
        let target = make_target(Site {
            bucket: "fake".to_string(),
            entry_point: None,
            include: None,
            exclude: None,
        });

        let test_dir = "test3";
        // If test dir already exists, delete it.
        if fs::metadata(test_dir).is_ok() {
            fs::remove_dir_all(test_dir).unwrap();
        }

        fs::create_dir(test_dir).unwrap();
        let test_pathname = format!("{}/notice_me.txt", test_dir);
        let test_path = PathBuf::from(&test_pathname);
        fs::File::create(&test_path).unwrap();

        let files: Vec<_> = get_dir_iterator(&target, Path::new(test_dir))
            .unwrap()
            .map(|entry| entry.unwrap().path().to_owned())
            .collect();

        assert!(files.contains(&test_path));

        fs::remove_dir_all(test_dir).unwrap();
    }

    #[test]
    fn it_can_filter_by_include() {
        let target = make_target(Site {
            bucket: "fake".to_string(),
            entry_point: None,
            include: Some(vec!["this_isnt_here.txt".to_string()]),
            exclude: None,
        });

        let test_dir = "test4";
        // If test dir already exists, delete it.
        if fs::metadata(test_dir).is_ok() {
            fs::remove_dir_all(test_dir).unwrap();
        }

        fs::create_dir(test_dir).unwrap();
        let test_pathname = format!("{}/ignore_me.txt", test_dir);
        let test_path = PathBuf::from(&test_pathname);
        fs::File::create(&test_path).unwrap();

        let files: Vec<_> = get_dir_iterator(&target, Path::new(test_dir))
            .unwrap()
            .map(|entry| entry.unwrap().path().to_owned())
            .collect();

        assert!(!files.contains(&test_path));

        fs::remove_dir_all(test_dir).unwrap();
    }

    #[test]
    fn it_can_filter_by_exclude() {
        let target = make_target(Site {
            bucket: "fake".to_string(),
            entry_point: None,
            include: None,
            exclude: Some(vec!["ignore_me.txt".to_string()]),
        });

        let test_dir = "test5";
        // If test dir already exists, delete it.
        if fs::metadata(test_dir).is_ok() {
            fs::remove_dir_all(test_dir).unwrap();
        }

        fs::create_dir(test_dir).unwrap();
        let test_pathname = format!("{}/ignore_me.txt", test_dir);
        let test_path = PathBuf::from(&test_pathname);
        fs::File::create(&test_path).unwrap();

        let files: Vec<_> = get_dir_iterator(&target, Path::new(test_dir))
            .unwrap()
            .map(|entry| entry.unwrap().path().to_owned())
            .collect();

        assert!(!files.contains(&test_path));

        fs::remove_dir_all(test_dir).unwrap();
    }

    #[test]
    fn it_can_prioritize_include_over_exclude() {
        let target = make_target(Site {
            bucket: "fake".to_string(),
            entry_point: None,
            include: Some(vec!["notice_me.txt".to_string()]),
            exclude: Some(vec!["notice_me.txt".to_string()]),
        });

        let test_dir = "test6";
        // If test dir already exists, delete it.
        if fs::metadata(test_dir).is_ok() {
            fs::remove_dir_all(test_dir).unwrap();
        }

        fs::create_dir(test_dir).unwrap();
        let test_pathname = format!("{}/notice_me.txt", test_dir);
        let test_path = PathBuf::from(&test_pathname);
        fs::File::create(&test_path).unwrap();

        let files: Vec<_> = get_dir_iterator(&target, Path::new(test_dir))
            .unwrap()
            .map(|entry| entry.unwrap().path().to_owned())
            .collect();

        assert!(files.contains(&test_path));

        fs::remove_dir_all(test_dir).unwrap();
    }

    #[test]
    fn it_inserts_hash_before_extension() {
        let value = "<h1>Hello World!</h1>";
        let hashed_value = get_digest(String::from(value)).unwrap();

        let path = PathBuf::from("path").join("to").join("asset.html");
        let actual_path_with_hash =
            generate_path_with_hash(&path, hashed_value.to_owned()).unwrap();

        let expected_path_with_hash = format!("path/to/asset.{}.html", hashed_value);

        assert_eq!(actual_path_with_hash, expected_path_with_hash);
    }

    #[test]
    fn it_inserts_hash_without_extension() {
        let value = "<h1>Hello World!</h1>";
        let hashed_value = get_digest(String::from(value)).unwrap();

        let path = PathBuf::from("path").join("to").join("asset");
        let actual_path_with_hash =
            generate_path_with_hash(&path, hashed_value.to_owned()).unwrap();

        let expected_path_with_hash = format!("path/to/asset.{}", hashed_value);

        assert_eq!(actual_path_with_hash, expected_path_with_hash);
    }

    #[test]
    fn it_generates_a_url_safe_hash() {
        let os_path = Path::new("some_stuff/invalid file&name.chars");
        let actual_url_safe_path = generate_url_safe_path(os_path).unwrap();
        // TODO: url-encode paths
        let expected_url_safe_path = "some_stuff/invalid file&name.chars";

        assert_eq!(actual_url_safe_path, expected_url_safe_path);
    }

    #[test]
    fn it_removes_bucket_dir_prefix() {
        let path = Path::new("./build/path/to/asset.ext");
        let directory = Path::new("./build");
        let value = Some("<h1>Hello World!</h1>".to_string());
        let (path, key) = generate_path_and_key(path, directory, value).unwrap();

        assert!(!path.contains("directory"));
        assert!(!key.contains("directory"));
    }

    #[test]
    fn it_combines_url_safe_and_hash_properly() {
        let path = Path::new("./build/path/to/asset.ext");
        let directory = Path::new("./build");
        let value = Some("<h1>Hello World!</h1>".to_string());
        let (path, key) = generate_path_and_key(path, directory, value).unwrap();

        let expected_path = "path/to/asset.ext".to_string();
        let expected_key_regex = Regex::new(r"^path/to/asset\.[0-9a-f]{64}\.ext").unwrap();

        assert_eq!(path, expected_path);
        assert!(expected_key_regex.is_match(&key));
    }
}
