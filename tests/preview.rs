#[macro_use]
extern crate lazy_static;

pub mod fixture;

use std::env;
use std::process::Command;
use std::sync::Mutex;

use assert_cmd::prelude::*;
use fixture::{rust, Fixture};

lazy_static! {
    static ref BUILD_LOCK: Mutex<u8> = Mutex::new(0);
}

#[test]
fn it_can_preview_js_project() {
    let fixture = Fixture::new("simple_js");
    fixture.create_file(
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
    fixture.create_default_package_json();
    fixture.create_wrangler_toml(
        r#"
        type = "javascript"
    "#,
    );
    preview_succeeds(&fixture);
    fixture.cleanup()
}

#[test]
fn it_can_preview_webpack_project() {
    let fixture = Fixture::new("webpack_simple_js");
    fixture.scaffold_webpack();
    fixture.create_wrangler_toml(
        r#"
        type = "webpack"
    "#,
    );
    preview_succeeds(&fixture);
    fixture.cleanup()
}

#[test]
fn it_can_preview_rust_project() {
    let fixture = Fixture::new("simple_rust");

    fixture.create_file("Cargo.toml", &rust::get_cargo_toml());

    fixture.create_dir("src");
    fixture.create_file("src/lib.rs", &rust::get_lib());
    fixture.create_file("src/utils.rs", &rust::get_utils());

    fixture.create_dir("worker");
    fixture.create_file(
        "worker/worker.js",
        r#"
        addEventListener('fetch', event => {
            event.respondWith(handleRequest(event.request))
        })

        /**
        * Fetch and log a request
        * @param {Request} request
        */
        async function handleRequest(request) {
            const { greet } = wasm_bindgen;
            await wasm_bindgen(wasm)
            const greeting = greet()
            return new Response(greeting, {status: 200})
        }
    "#,
    );

    fixture.create_wrangler_toml(
        r#"
        type = "rust"
    "#,
    );
    preview_succeeds(&fixture);
    fixture.cleanup()
}

fn preview_succeeds(fixture: &Fixture) {
    // Lock to avoid having concurrent builds
    let _g = BUILD_LOCK.lock().unwrap();
    env::remove_var("CF_ACCOUNT_ID");
    let mut preview = Command::cargo_bin(env!("CARGO_PKG_NAME")).unwrap();
    preview.current_dir(fixture.get_path());
    preview.arg("preview").arg("--headless").assert().success();
}
