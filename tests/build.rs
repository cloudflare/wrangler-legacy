use assert_cmd::prelude::*;
use std::process::Command;
use std::str;

mod test_utils;

#[test]
fn it_builds_with_webpack_single_js() {
    let fixture = "webpack_simple_js";
    test_utils::create_temporary_copy(fixture);

    test_utils::webpack_config(
        fixture,
        r#"{
            entry: "./index.js",
        }"#,
    );
    test_utils::wrangler_config(
        fixture,
        r#"
        type = "Webpack"
    "#,
    );

    build(fixture);
    assert!(test_utils::fixture_out_path(fixture)
        .join("script.js")
        .exists());
    test_utils::cleanup(fixture);
}

#[test]
fn it_builds_with_webpack_single_js_use_package_main() {
    let fixture = "webpack_single_js_use_package_main";
    test_utils::create_temporary_copy(fixture);

    test_utils::wrangler_config(
        fixture,
        r#"
        type = "Webpack"
    "#,
    );

    build(fixture);
    assert!(test_utils::fixture_out_path(fixture)
        .join("script.js")
        .exists());
    test_utils::cleanup(fixture);
}

#[test]
fn it_builds_with_webpack_specify_configs() {
    let fixture = "webpack_specify_config";
    test_utils::create_temporary_copy(fixture);

    test_utils::wrangler_config(
        fixture,
        r#"
        type = "Webpack"
        webpack_config = "webpack.worker.js"
    "#,
    );

    build(fixture);
    assert!(test_utils::fixture_out_path(fixture)
        .join("script.js")
        .exists());
    test_utils::cleanup(fixture);
}

#[test]
fn it_builds_with_webpack_single_js_missing_package_main() {
    let fixture = "webpack_single_js_missing_package_main";
    test_utils::create_temporary_copy(fixture);

    test_utils::wrangler_config(
        fixture,
        r#"
        type = "Webpack"
    "#,
    );

    build_fails_with(
        fixture,
        "The `main` key in your `package.json` file is required",
    );
    test_utils::cleanup(fixture);
}

#[test]
fn it_fails_with_multiple_webpack_configs() {
    let fixture = "webpack_multiple_config";
    test_utils::create_temporary_copy(fixture);

    test_utils::webpack_config(
        fixture,
        r#"[
          { entry: "./a.js" },
          { entry: "./b.js" }
        ]"#,
    );
    test_utils::wrangler_config(
        fixture,
        r#"
        type = "Webpack"
    "#,
    );

    build_fails_with(fixture, "Multiple webpack configurations are not supported. You can specify a different path for your webpack configuration file in wrangler.toml with the `webpack_config` field");
    test_utils::cleanup(fixture);
}

#[test]
fn it_builds_with_webpack_wast() {
    let fixture = "webpack_wast";
    test_utils::create_temporary_copy(fixture);

    test_utils::wrangler_config(
        fixture,
        r#"
        type = "Webpack"
    "#,
    );

    build(fixture);
    assert!(test_utils::fixture_out_path(fixture)
        .join("script.js")
        .exists());
    assert!(test_utils::fixture_out_path(fixture)
        .join("module.wasm")
        .exists());

    test_utils::cleanup(fixture);
}

fn build(fixture: &str) {
    let mut build = Command::cargo_bin(env!("CARGO_PKG_NAME")).unwrap();
    build.current_dir(test_utils::fixture_path(fixture));
    build.arg("build").assert().success();
}

fn build_fails_with(fixture: &str, expected_message: &str) {
    let mut build = Command::cargo_bin(env!("CARGO_PKG_NAME")).unwrap();
    build.current_dir(test_utils::fixture_path(fixture));
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
