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

macro_rules! single_env_settings {
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
fn it_builds_with_webpack_single_js() {
    let fixture = "webpack_simple_js";
    utils::create_temporary_copy(fixture);
    single_env_settings! {fixture, r#"
        type = "Webpack"
    "#};

    build(fixture);
    assert!(utils::fixture_out_path(fixture).join("script.js").exists());
    utils::cleanup(fixture);
}

#[test]
fn it_builds_with_webpack_function_config_js() {
    let fixture = "webpack_function_config_js";
    utils::create_temporary_copy(fixture);

    single_env_settings! {fixture, r#"
        type = "Webpack"
    "#};

    build(fixture);
    assert!(utils::fixture_out_path(fixture).join("script.js").exists());
    utils::cleanup(fixture);
}

#[test]
fn it_builds_with_webpack_promise_config_js() {
    let fixture = "webpack_promise_config_js";
    utils::create_temporary_copy(fixture);

    single_env_settings! {fixture, r#"
        type = "Webpack"
    "#};

    build(fixture);
    assert!(utils::fixture_out_path(fixture).join("script.js").exists());
    utils::cleanup(fixture);
}

#[test]
fn it_builds_with_webpack_function_promise_config_js() {
    let fixture = "webpack_function_promise_config_js";
    utils::create_temporary_copy(fixture);

    single_env_settings! {fixture, r#"
        type = "Webpack"
    "#};

    build(fixture);
    assert!(utils::fixture_out_path(fixture).join("script.js").exists());
    utils::cleanup(fixture);
}

#[test]
fn it_builds_with_webpack_single_js_use_package_main() {
    let fixture = "webpack_single_js_use_package_main";
    utils::create_temporary_copy(fixture);

    single_env_settings! {fixture, r#"
        type = "Webpack"
    "#};

    build(fixture);
    assert!(utils::fixture_out_path(fixture).join("script.js").exists());
    utils::cleanup(fixture);
}

#[test]
fn it_builds_with_webpack_specify_configs() {
    let fixture = "webpack_specify_config";
    utils::create_temporary_copy(fixture);

    single_env_settings! {fixture, r#"
        type = "Webpack"
        webpack_config = "webpack.worker.js"
    "#};

    build(fixture);
    assert!(utils::fixture_out_path(fixture).join("script.js").exists());
    utils::cleanup(fixture);
}

#[test]
fn it_builds_with_webpack_single_js_missing_package_main() {
    let fixture = "webpack_single_js_missing_package_main";
    utils::create_temporary_copy(fixture);

    single_env_settings! {fixture, r#"
        type = "Webpack"
    "#};

    build_fails_with(
        fixture,
        "The `main` key in your `package.json` file is required",
    );
    utils::cleanup(fixture);
}

#[test]
fn it_fails_with_multiple_webpack_configs() {
    let fixture = "webpack_multiple_config";
    utils::create_temporary_copy(fixture);

    single_env_settings! {fixture, r#"
        type = "Webpack"
    "#};

    build_fails_with(fixture, "Multiple webpack configurations are not supported. You can specify a different path for your webpack configuration file in wrangler.toml with the `webpack_config` field");
    utils::cleanup(fixture);
}

#[test]
fn it_fails_with_multiple_specify_webpack_configs() {
    let fixture = "webpack_multiple_specify_config";
    utils::create_temporary_copy(fixture);

    single_env_settings! {fixture, r#"
        type = "Webpack"
        webpack_config = "webpack.worker.js"
    "#};

    build_fails_with(fixture, "Multiple webpack configurations are not supported. You can specify a different path for your webpack configuration file in wrangler.toml with the `webpack_config` field");
    utils::cleanup(fixture);
}

#[test]
fn it_builds_with_webpack_wast() {
    let fixture = "webpack_wast";
    utils::create_temporary_copy(fixture);

    single_env_settings! {fixture, r#"
        type = "Webpack"
    "#};

    build(fixture);
    assert!(utils::fixture_out_path(fixture).join("script.js").exists());
    assert!(utils::fixture_out_path(fixture)
        .join("module.wasm")
        .exists());

    utils::cleanup(fixture);
}

#[test]
fn it_fails_with_webpack_target_node() {
    let fixture = "webpack_target_node";
    utils::create_temporary_copy(fixture);

    utils::webpack_config(
        fixture,
        r#"{
          entry: "./index.js",
          target: "node",
        }"#,
    );
    single_env_settings! {fixture, r#"
        type = "webpack"
    "#};

    build_fails_with(
        fixture,
        "Building a Cloudflare Worker with target \"node\" is not supported",
    );
    utils::cleanup(fixture);
}

#[test]
fn it_fails_with_webpack_target_web() {
    let fixture = "webpack_target_web";
    utils::create_temporary_copy(fixture);

    utils::webpack_config(
        fixture,
        r#"{
          entry: "./index.js",
          target: "web",
        }"#,
    );
    single_env_settings! {fixture, r#"
        type = "webpack"
    "#};

    build_fails_with(
        fixture,
        "Building a Cloudflare Worker with target \"web\" is not supported",
    );
    utils::cleanup(fixture);
}

#[test]
fn it_builds_with_webpack_target_webworker() {
    let fixture = "webpack_target_webworker";
    utils::create_temporary_copy(fixture);

    utils::webpack_config(
        fixture,
        r#"{
          entry: "./index.js",
          target: "webworker",
        }"#,
    );
    single_env_settings! {fixture, r#"
        type = "webpack"
    "#};

    build(fixture);
    utils::cleanup(fixture);
}

fn build(fixture: &str) {
    // Lock to avoid having concurrent builds
    let _g = BUILD_LOCK.lock().unwrap();

    let mut build = Command::cargo_bin(env!("CARGO_PKG_NAME")).unwrap();
    build.current_dir(utils::fixture_path(fixture));
    build.arg("build").assert().success();
}

fn build_fails_with(fixture: &str, expected_message: &str) {
    // Lock to avoid having concurrent builds
    let _g = BUILD_LOCK.lock().unwrap();

    let mut build = Command::cargo_bin(env!("CARGO_PKG_NAME")).unwrap();
    build.current_dir(utils::fixture_path(fixture));
    build.arg("build");

    let output = build.output().expect("failed to execute process");
    assert!(!output.status.success());
    assert!(
        str::from_utf8(&output.stderr)
            .unwrap()
            .contains(expected_message),
        format!(
            "expected {:?} not found, given: {:?}",
            expected_message,
            str::from_utf8(&output.stderr)
        )
    );
}
