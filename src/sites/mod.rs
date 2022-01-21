extern crate base64;

mod manifest;
mod sync;

pub use manifest::AssetManifest;
pub use sync::sync;

use std::collections::HashSet;
use std::error::Error;
use std::ffi::OsString;
use std::fmt;
use std::fs;
use std::hash::Hasher;
use std::path::Path;

use anyhow::{anyhow, Result};
use ignore::overrides::{Override, OverrideBuilder};
use ignore::{Walk, WalkBuilder};
use indicatif::{ProgressBar, ProgressStyle};
use twox_hash::XxHash64;

use cloudflare::endpoints::workerskv::write_bulk::KeyValuePair;

use crate::kv::namespace::{upsert, UpsertedNamespace};
use crate::settings::global_user::GlobalUser;
use crate::settings::toml::{KvNamespace, Target};
use crate::terminal::message::{Message, StdErr};
pub const KEY_MAX_SIZE: usize = 512;
// Oddly enough, metadata.len() returns a u64, not usize.
pub const VALUE_MAX_SIZE: u64 = 25 * 1024 * 1024;

// Updates given Target with kv_namespace binding for a static site assets KV namespace.
pub fn add_namespace(user: &GlobalUser, target: &mut Target, preview: bool) -> Result<KvNamespace> {
    let title = if preview {
        format!("__{}-{}", target.name, "workers_sites_assets_preview")
    } else {
        format!("__{}-{}", target.name, "workers_sites_assets")
    };

    let site_namespace = match upsert(target, user, title)? {
        UpsertedNamespace::Created(namespace) => {
            let msg = format!("Created namespace for Workers Site \"{}\"", namespace.title);
            StdErr::working(&msg);

            namespace
        }
        UpsertedNamespace::Reused(namespace) => {
            let msg = format!("Using namespace for Workers Site \"{}\"", namespace.title);
            StdErr::working(&msg);

            namespace
        }
    };

    let site_namespace = KvNamespace {
        binding: "__STATIC_CONTENT".to_string(),
        id: site_namespace.id,
    };

    target.add_kv_namespace(site_namespace.clone());

    Ok(site_namespace)
}

#[derive(Debug, Clone)]
pub struct NotADirectoryError;

impl fmt::Display for NotADirectoryError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Not a directory. Check your configuration file; the `bucket` attribute for [site] should point to a directory.")
    }
}

impl Error for NotADirectoryError {}

// Returns the hashed key and value pair for all files in a directory.
pub fn directory_keys_values(
    target: &Target,
    directory: &Path,
    exclude: Option<&HashSet<String>>,
) -> Result<(Vec<KeyValuePair>, AssetManifest, Vec<String>)> {
    match fs::metadata(directory) {
        Ok(ref file_type) if file_type.is_dir() => {
            let mut upload_vec: Vec<KeyValuePair> = Vec::new();
            let mut asset_manifest = AssetManifest::new();
            let mut file_list: Vec<String> = Vec::new();
            let dir_walker = get_dir_iterator(target, directory)?;
            let spinner_style =
                ProgressStyle::default_spinner().template("{spinner}   Preparing {msg}...");
            let spinner = ProgressBar::new_spinner().with_style(spinner_style);

            for entry in dir_walker {
                spinner.tick();
                let entry = entry.unwrap();
                let path = entry.path();
                if path.is_file() {
                    spinner.set_message(&format!("{}", path.display()));

                    file_list.push(path.to_str().unwrap().to_string());
                    validate_file_size(path)?;

                    let value = std::fs::read(path)?;

                    // Need to base64 encode value
                    let b64_value = base64::encode(&value);

                    let (url_safe_path, key) =
                        generate_path_and_key(path, directory, Some(b64_value.clone()))?;

                    validate_key_size(&key)?;

                    // asset manifest should always contain all files
                    asset_manifest.insert(url_safe_path, key.clone());

                    // skip uploading existing keys, if configured to do so
                    if let Some(remote_keys) = exclude {
                        if remote_keys.contains(&key) {
                            continue;
                        }
                    }

                    upload_vec.push(KeyValuePair {
                        key: key.clone(),
                        value: b64_value,
                        expiration: None,
                        expiration_ttl: None,
                        base64: Some(true),
                    });
                }
            }
            Ok((upload_vec, asset_manifest, file_list))
        }
        Ok(_file_type) => {
            // any other file types (files, symlinks)
            Err(anyhow::Error::new(NotADirectoryError))
        }
        Err(e) => Err(anyhow!(e)),
    }
}

// Ensure that all files in upload directory do not exceed the MAX_VALUE_SIZE (this ensures that
// no partial uploads happen). I don't like this functionality (and the similar key length checking
// logic in validate_key_size()) because it duplicates the size checking the API already does--but
// doing a preemptive check like this (before calling the API) will prevent partial bucket uploads
// from happening.
fn validate_file_size(path: &Path) -> Result<()> {
    let metadata = fs::metadata(path)?;
    let file_len = metadata.len();

    if file_len > VALUE_MAX_SIZE {
        anyhow::bail!(
            "File `{}` of {} bytes exceeds the maximum value size limit of {} bytes",
            path.display(),
            file_len,
            VALUE_MAX_SIZE
        );
    }
    Ok(())
}

fn validate_key_size(key: &str) -> Result<()> {
    if key.len() > KEY_MAX_SIZE {
        anyhow::bail!(
            "Path `{}` of {} bytes exceeds the maximum key size limit of {} bytes",
            key,
            key.len(),
            KEY_MAX_SIZE
        );
    }
    Ok(())
}

const REQUIRED_IGNORE_FILES: &[&str] = &[NODE_MODULES];
const NODE_MODULES: &str = "node_modules";

fn get_dir_iterator(target: &Target, directory: &Path) -> Result<Walk> {
    // The directory provided should never be node_modules!
    if let Some(name) = directory.file_name() {
        if name == NODE_MODULES {
            anyhow::bail!("Your directory of files to upload cannot be named node_modules.");
        }
    };

    let ignore = build_ignore(target, directory)?;
    Ok(WalkBuilder::new(directory)
        .standard_filters(false)
        .overrides(ignore)
        .build())
}

fn build_ignore(target: &Target, directory: &Path) -> Result<Override> {
    let mut required_override = OverrideBuilder::new(directory);
    let required_ignore = |builder: &mut OverrideBuilder| -> Result<()> {
        for ignored in REQUIRED_IGNORE_FILES {
            builder.add(&format!("!{}", ignored))?;
            log::info!("Ignoring {}", ignored);
        }

        Ok(())
    };
    if let Some(site) = &target.site {
        // If `include` present, use it and don't touch the `exclude` field
        if let Some(included) = &site.include {
            required_ignore(&mut required_override)?;
            for i in included {
                required_override.add(i)?;
                log::info!("Including {}", i);
            }
        } else {
            // allow all files. This is required since without this the `.well-known`
            // override would act as a allowlist
            required_override.add("*")?;
            // ignore hidden files and folders
            required_override.add("!.*")?;
            // but allow .well-known, this has precedence over hidden files since it's later
            required_override.add(".well-known")?;
            // add this AFTER since the `*` override would have precedence over this,
            // making it useless
            required_ignore(&mut required_override)?;

            // add any other excludes specified
            if let Some(excluded) = &site.exclude {
                for e in excluded {
                    required_override.add(&format!("!{}", e))?;
                    log::info!("Ignoring {}", e);
                }
            }
        }
    }

    let exclude = required_override.build()?;
    Ok(exclude)
}

// Courtesy of Steve Klabnik's PoC :) Used for bulk operations (write, delete)
fn generate_url_safe_path(path: &Path) -> Result<String> {
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

// Adds the XXhash hash of the path's file contents to the url-safe path of a file to
// generate a versioned key for the file and its contents. Returns the url-safe path prefix
// for the key, as well as the key with hash appended.
// e.g (sitemap.xml, sitemap.ec717eb213.xml)
pub fn generate_path_and_key(
    path: &Path,
    directory: &Path,
    value: Option<String>,
) -> Result<(String, String)> {
    // strip the bucket directory from both paths for ease of reference.
    let relative_path = path.strip_prefix(directory).unwrap();

    let url_safe_path = generate_url_safe_path(relative_path)?;
    let path_with_hash = if let Some(value) = value {
        let digest = get_digest(value);
        // it is ok to truncate the digest here because
        // we also include the file name in the asset manifest key
        //
        // the most important thing here is to detect changes
        // of a single file to invalidate the cache and
        // it's impossible to serve two different files with the same name
        let digest = digest[0..10].to_string();
        generate_path_with_hash(relative_path, digest)?
    } else {
        url_safe_path.to_owned()
    };

    Ok((url_safe_path, path_with_hash))
}

fn get_digest(value: String) -> String {
    let value = value.as_bytes();
    let mut hasher = XxHash64::default();
    hasher.write(value);
    let digest = hasher.finish();
    // encode u64 as hexadecimal to save space and information
    format!("{:x}", digest)
}

// Assumes that `path` is a file (called from a match branch for path.is_file())
// Assumes that `hashed_value` is a String, not an Option<String> (called from a match branch for value.is_some())
fn generate_path_with_hash(path: &Path, hashed_value: String) -> Result<String> {
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
        anyhow::bail!("no file_stem for path {}", path.display())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use regex::Regex;
    use std::fs;
    use std::io::Write;
    use std::path::{Path, PathBuf};
    use tempfile::TempDir;

    use crate::settings::toml::{Site, Target, TargetType};

    fn make_target(site: Site) -> Target {
        Target {
            account_id: None.into(),
            kv_namespaces: Vec::new(),
            r2_buckets: Vec::new(),
            durable_objects: None,
            migrations: None,
            name: "".to_string(),
            target_type: TargetType::JavaScript,
            webpack_config: None,
            site: Some(site),
            build: None,
            vars: None,
            text_blobs: None,
            usage_model: None,
            wasm_modules: None,
            compatibility_date: None,
            compatibility_flags: Vec::new(),
        }
    }

    fn tmpdir_with_default_files() -> (PathBuf, Vec<PathBuf>) {
        let files = vec![
            PathBuf::new().join("file_a.txt"),
            PathBuf::new().join("file_b.txt"),
            PathBuf::new().join("file_c.txt"),
        ];

        let tmpdir = TempDir::new().unwrap();
        let tmp_path = tmpdir.into_path();

        files.iter().for_each(|path| {
            std::fs::File::create(tmp_path.clone().join(path)).unwrap();
        });

        (tmp_path, files)
    }

    #[test]
    fn it_runs_directory_keys_values_returning_expected_valyes() {
        let (tmpdir, all_files) = tmpdir_with_default_files();

        // check that no files are excluded from the upload set or the asset manifest.
        let (to_upload, asset_manifest, _) =
            directory_keys_values(&make_target(Site::default()), &tmpdir, None).unwrap();
        let mut keys = vec![];
        for file in &all_files {
            let filename = file.to_str().unwrap();
            assert!(asset_manifest.get(filename).is_some());
            keys.push(asset_manifest.get(filename).unwrap());
        }
        for kv in to_upload {
            assert!(keys.contains(&&kv.key))
        }

        // check that the correct files are excluded (as if they already are uploaded).
        // asset manifest should have all the files, to_upload should be filtered.
        let mut exclude = HashSet::new();
        for f in &["file_a.txt", "file_b.txt"] {
            let path = tmpdir.join(f);
            let path = path.to_str().unwrap();
            // in calling code, `exclude` is the list of keys from KV, and thus needs to contain the
            // partial hash digest of the file in the "key". call generate_path_and_key to obtain
            // for later comparison.
            let (_, key_with_hash) =
                generate_path_and_key(Path::new(path), &tmpdir, Some("".into())).unwrap();
            exclude.insert(key_with_hash);
        }

        let (to_upload, asset_manifest, _) =
            directory_keys_values(&make_target(Site::default()), &tmpdir, Some(&exclude)).unwrap();
        for file in &all_files {
            let filename = file.to_str().unwrap();
            assert!(asset_manifest.get(filename).is_some());
        }
        // construct a list of keys from the upload list to then ensure it does _not_ contain any
        // key for a file which we have designated for exclusion.
        let upload_keys: Vec<String> = to_upload.iter().map(|kv| kv.key.clone()).collect();
        for file in exclude.iter() {
            assert!(!upload_keys.contains(&file.to_string()));
        }
        assert_eq!(upload_keys.len(), 1);
        assert!(upload_keys.first().unwrap().starts_with("file_c"));
        assert_eq!(to_upload.len(), all_files.len() - exclude.len());
    }

    #[test]
    fn it_can_ignore_node_modules() {
        let mut site = Site::default();
        site.bucket = PathBuf::from("fake");
        let target = make_target(site);

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
    fn it_can_ignore_hidden_except_wellknown() {
        let mut site = Site::default();
        site.bucket = PathBuf::from("fake");
        let target = make_target(site);

        let test_dir = "test2";
        // If test dir already exists, delete it.
        if fs::metadata(test_dir).is_ok() {
            fs::remove_dir_all(test_dir).unwrap();
        }

        fs::create_dir(test_dir).unwrap();
        fs::create_dir(format!("{}/.well-known", test_dir)).unwrap();
        fs::File::create(&PathBuf::from(&format!("{}/.ignore_me.txt", test_dir))).unwrap();
        fs::File::create(&PathBuf::from(&format!(
            "{}/.well-known/dontignoreme.txt",
            test_dir
        )))
        .unwrap();
        let (_, _, file_list) = directory_keys_values(&target, Path::new(test_dir), None).unwrap();
        if cfg!(windows) {
            assert!(!file_list.contains(&format!("{}\\.ignore_me.txt", test_dir)));
            assert!(file_list.contains(&format!("{}\\.well-known\\dontignoreme.txt", test_dir)));
        } else {
            assert!(!file_list.contains(&format!("{}/.ignore_me.txt", test_dir)));
            assert!(file_list.contains(&format!("{}/.well-known/dontignoreme.txt", test_dir)));
        }

        fs::remove_dir_all(test_dir).unwrap();
    }

    #[test]
    fn it_can_allow_unfiltered_files() {
        let mut site = Site::default();
        site.bucket = PathBuf::from("fake");
        let target = make_target(site);

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
        let mut site = Site::default();
        site.bucket = PathBuf::from("fake");
        site.include = Some(vec!["this_isnt_here.txt".to_string()]);
        let target = make_target(site);

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
        let mut site = Site::default();
        site.bucket = PathBuf::from("fake");
        site.exclude = Some(vec!["ignore_me.txt".to_string()]);
        let target = make_target(site);

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
        let mut site = Site::default();
        site.bucket = PathBuf::from("fake");
        site.include = Some(vec!["notice_me.txt".to_string()]);
        site.exclude = Some(vec!["notice_me.txt".to_string()]);
        let target = make_target(site);

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
    fn it_can_include_gitignore_entries() {
        // We don't want our wrangler include/exclude functionality to read .gitignore files.
        let mut site = Site::default();
        site.bucket = PathBuf::from("public");
        let target = make_target(site);

        let test_dir = "test7";
        // If test dir already exists, delete it.
        if fs::metadata(test_dir).is_ok() {
            fs::remove_dir_all(test_dir).unwrap();
        }

        fs::create_dir(test_dir).unwrap();
        // Create .gitignore file in test7 directory
        let gitignore_pathname = format!("{}/.gitignore", test_dir);
        let gitignore_path = PathBuf::from(&gitignore_pathname);
        let mut gitignore = fs::File::create(&gitignore_path).unwrap();
        writeln!(gitignore, "public/").unwrap();

        // Create 'public/' directory, which should be included.
        let upload_dir = format!("{}/public", test_dir);
        fs::create_dir(&upload_dir).unwrap();
        let test_pathname = format!("{}/notice_me.txt", &upload_dir);
        let test_path = PathBuf::from(&test_pathname);
        fs::File::create(&test_path).unwrap();

        let files: Vec<_> = get_dir_iterator(&target, Path::new(&upload_dir))
            .unwrap()
            .map(|entry| entry.unwrap().path().to_owned())
            .collect();

        assert!(files.contains(&test_path));

        // Why drop()? Well, fs::remove_dir_all on Windows depends on the DeleteFileW syscall.
        // This syscall doesn't actually delete a file, but only marks it for deletion. It still
        // can be alive when we try to delete the parent directory test_dir, causing a "directory
        // is not empty" error. As a result, we MUST call drop() to close the gitignore file so
        // that it is not alive when fs::remove_dir_all(test_dir) is called.
        drop(gitignore);
        fs::remove_dir_all(test_dir).unwrap();
    }

    #[test]
    fn it_inserts_hash_before_extension() {
        let value = "<h1>Hello World!</h1>";
        let hashed_value = get_digest(String::from(value));

        let path = PathBuf::from("path").join("to").join("asset.html");
        let actual_path_with_hash =
            generate_path_with_hash(&path, hashed_value.to_owned()).unwrap();

        let expected_path_with_hash = format!("path/to/asset.{}.html", hashed_value);

        assert_eq!(actual_path_with_hash, expected_path_with_hash);
    }

    #[test]
    fn it_inserts_hash_without_extension() {
        let value = "<h1>Hello World!</h1>";
        let hashed_value = get_digest(String::from(value));

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
        let expected_key_regex = Regex::new(r"^path/to/asset\.[0-9a-f]{10}\.ext").unwrap();

        assert_eq!(path, expected_path);
        assert!(expected_key_regex.is_match(&key));
    }
}
