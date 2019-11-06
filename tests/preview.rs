#[macro_use]
extern crate lazy_static;

pub mod utils;

use assert_cmd::prelude::*;

use std::env;
use std::process::Command;
use std::str;
use std::sync::Mutex;

lazy_static! {
    static ref BUILD_LOCK: Mutex<u8> = Mutex::new(0);
}

#[test]
fn it_can_preview_js_project() {
    let fixture = "simple_js";
    utils::create_temporary_copy(fixture);
    utils::create_fixture_file(
        fixture,
        "index.js",
        r#"
        addEventListener('fetch', event => {
            event.respondWith(handleRequest(event.request))
        })
        
        /**
        * Fetch and log a request
        * @param {Request} request
        */
        async function handleRequest(request) {
            return new Response('Hello worker!', { status: 200 })
        }
    "#,
    );
    utils::create_default_package_json(fixture);
    utils::create_wrangler_toml(
        fixture,
        r#"
        type = "javascript"
    "#,
    );
    preview(fixture);
    utils::cleanup(fixture);
}

#[test]
fn it_can_preview_webpack_project() {
    let fixture = "webpack_simple_js";
    utils::create_temporary_copy(fixture);
    utils::create_default_package_json(fixture);
    utils::create_empty_js(fixture);
    utils::create_wrangler_toml(
        fixture,
        r#"
        type = "webpack"
    "#,
    );
    preview(fixture);
    utils::cleanup(fixture);
}

#[test]
fn it_can_preview_rust_project() {
    let fixture = "simple_rust";
    utils::create_temporary_copy(fixture);
    utils::create_empty_package_json(fixture);
    utils::create_wrangler_toml(
        fixture,
        r#"
        type = "rust"
    "#,
    );
    preview(fixture);
    utils::cleanup(fixture);
}

fn preview(fixture: &str) {
    // Lock to avoid having concurrent builds
    let _g = BUILD_LOCK.lock().unwrap();
    env::remove_var("CF_ACCOUNT_ID");
    let mut preview = Command::cargo_bin(env!("CARGO_PKG_NAME")).unwrap();
    preview.current_dir(utils::fixture_path(fixture));
    preview.arg("preview").arg("--headless").assert().success();
}
