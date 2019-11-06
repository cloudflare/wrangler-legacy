use fs_extra::dir::{copy, CopyOptions};
use std::env;
use std::fs;
use std::fs::File;
use std::io::prelude::*;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::Mutex;

lazy_static! {
    static ref BUILD_LOCK: Mutex<u8> = Mutex::new(0);
}

const BUNDLE_OUT: &str = "./worker";

pub fn cleanup(fixture: &str) {
    let path = fixture_path(fixture);
    assert!(path.exists(), format!("{:?} does not exist", path));

    // Workaround https://github.com/rust-lang/rust/issues/29497
    if cfg!(target_os = "windows") {
        let mut command = Command::new("cmd");
        command.arg("rmdir");
        command.arg("/s");
        command.arg(&path);
    } else {
        fs::remove_dir_all(&path).unwrap();
    }
}

pub fn fixture_path(fixture: &str) -> PathBuf {
    let mut dest = env::temp_dir();
    dest.push(fixture);
    dest
}

pub fn fixture_out_path(fixture: &str) -> PathBuf {
    fixture_path(fixture).join(BUNDLE_OUT)
}

pub fn create_temporary_copy(fixture: &str) {
    let current_dir = env::current_dir().unwrap();
    let src = Path::new(&current_dir).join("tests/fixtures").join(fixture);

    let dest = env::temp_dir();

    if dest.join(fixture).exists() {
        cleanup(fixture);
    }

    fs::create_dir_all(dest.clone()).unwrap();
    let mut options = CopyOptions::new();
    options.overwrite = true;
    copy(src, dest, &options).unwrap();
}

pub fn create_fixture_file(fixture: &str, name: &str, content: &str) {
    let file_path = fixture_path(fixture).join(name);
    println!("{:?}", file_path);
    let mut file = File::create(file_path).unwrap();
    let content = String::from(content);
    file.write_all(content.as_bytes()).unwrap();
}

pub fn create_empty_package_json(fixture: &str) {
    create_fixture_file(fixture, "package.json", "{}");
}

pub fn create_default_package_json(fixture: &str) {
    create_fixture_file(
        fixture,
        "package.json",
        r#"
        {
            "main": "index.js"
        }
    "#,
    );
}

pub fn create_empty_js(fixture: &str) {
    create_fixture_file(fixture, "index.js", "// js")
}

pub fn create_wrangler_toml(fixture: &str, extra_fields: &str) {
    let content = &format!(
        r#"
            name = "test"
            workers_dev = true
            {}
        "#,
        extra_fields
    );
    create_fixture_file(fixture, "wrangler.toml", content);
}
