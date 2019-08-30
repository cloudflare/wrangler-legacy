use assert_cmd::prelude::*;
use std::process::Command;

mod test_utils;

#[test]
fn it_can_preview_js_project() {
    let fixture = "simple_js";
    test_utils::create_temporary_copy(fixture);
    test_utils::wrangler_config(
        fixture,
        r#"
        type = "javascript"
    "#,
    );
    preview(fixture);
    test_utils::cleanup(fixture);
}

#[test]
fn it_can_preview_webpack_project() {
    let fixture = "webpack_simple_js";
    test_utils::create_temporary_copy(fixture);
    test_utils::wrangler_config(
        fixture,
        r#"
        type = "webpack"
    "#,
    );
    preview(fixture);
    test_utils::cleanup(fixture);
}

#[test]
fn it_can_preview_rust_project() {
    let fixture = "simple_rust";
    test_utils::create_temporary_copy(fixture);
    test_utils::wrangler_config(
        fixture,
        r#"
        type = "rust"
    "#,
    );
    preview(fixture);
    test_utils::cleanup(fixture);
}

fn preview(fixture: &str) {
    let mut preview = Command::cargo_bin(env!("CARGO_PKG_NAME")).unwrap();
    preview.current_dir(test_utils::fixture_path(fixture));
    preview.arg("preview").assert().success();
}
