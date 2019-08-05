use assert_cmd::prelude::*;
use fs_extra::dir::{copy, CopyOptions};
use std::env;
use std::fs;
use std::fs::File;
use std::io::prelude::*;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::str;

macro_rules! settings {
    ( $f:expr, $x:expr ) => {
        let file_path = fixture_path($f).join("wrangler.toml");
        let mut file = File::create(file_path).unwrap();
        let content = format!(
            r#"
            name = "test"
            zone_id = ""
            account_id = ""
            {}
        "#,
            $x
        );
        file.write_all(content.as_bytes()).unwrap();
    };
}

#[test]
fn it_can_preview_js_project() {
    let fixture = "simple_js";
    create_temporary_copy(fixture);
    settings! {fixture, r#"
        type = "javascript"
    "#};
    preview(fixture);
    cleanup(fixture);
}

#[test]
fn it_can_preview_webpack_project() {
    let fixture = "webpack_simple_js";
    create_temporary_copy(fixture);
    settings! {fixture, r#"
        type = "webpack"
    "#};
    preview(fixture);
    cleanup(fixture);
}

#[test]
fn it_can_preview_rust_project() {
    let fixture = "simple_rust";
    create_temporary_copy(fixture);
    settings! {fixture, r#"
        type = "rust"
    "#};
    preview(fixture);
    cleanup(fixture);
}

fn preview(fixture: &str) {
    let mut preview = Command::cargo_bin(env!("CARGO_PKG_NAME")).unwrap();
    preview.current_dir(fixture_path(fixture));
    preview.arg("preview").assert().success();
}

fn cleanup(fixture: &str) {
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

fn fixture_path(fixture: &str) -> PathBuf {
    let mut dest = env::temp_dir();
    dest.push(fixture);
    dest
}

fn create_temporary_copy(fixture: &str) {
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
