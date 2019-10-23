#[macro_use]
extern crate lazy_static;

mod utils;

use assert_cmd::prelude::*;

use std::fs::File;
use std::io::Write;
use std::process::Command;
use std::str;
use std::sync::Mutex;

lazy_static! {
    static ref BUILD_LOCK: Mutex<u8> = Mutex::new(0);
}

macro_rules! settings {
    ( $f:expr, $x:expr ) => {
        let file_path = utils::fixture_path($f).join("wrangler.toml");
        let mut file = File::create(file_path).unwrap();
        let content = format!(
            r#"
            name = "test"
            zone_id = ""
            account_id = ""
            workers_dev = true
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
    utils::create_temporary_copy(fixture);
    settings! {fixture, r#"
        type = "javascript"
    "#};
    preview(fixture);
    utils::cleanup(fixture);
}

#[test]
fn it_can_preview_webpack_project() {
    let fixture = "webpack_simple_js";
    utils::create_temporary_copy(fixture);
    settings! {fixture, r#"
        type = "webpack"
    "#};
    preview(fixture);
    utils::cleanup(fixture);
}

#[test]
fn it_can_preview_rust_project() {
    let fixture = "simple_rust";
    utils::create_temporary_copy(fixture);
    settings! {fixture, r#"
        type = "rust"
    "#};
    preview(fixture);
    utils::cleanup(fixture);
}

fn preview(fixture: &str) {
    // Lock to avoid having concurrent builds
    let _g = BUILD_LOCK.lock().unwrap();

    let mut preview = Command::cargo_bin(env!("CARGO_PKG_NAME")).unwrap();
    preview.current_dir(utils::fixture_path(fixture));
    preview.arg("preview").assert().success();
}
