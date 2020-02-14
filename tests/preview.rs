#[macro_use]
extern crate lazy_static;

pub mod fixture;

use fixture::{EnvConfig, WranglerToml, TEST_ENV_NAME};

use std::collections::HashMap;
use std::env;
use std::process::Command;

use assert_cmd::prelude::*;
use fixture::{rust, Fixture};

#[test]
fn it_can_preview_js_project() {
    let fixture = Fixture::new();
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

    let wrangler_toml = WranglerToml::javascript("test-preview-javascript");
    fixture.create_wrangler_toml(wrangler_toml);

    preview_succeeds(&fixture);
}

#[test]
fn it_can_preview_webpack_project() {
    let fixture = Fixture::new();
    fixture.scaffold_webpack();

    let wrangler_toml = WranglerToml::webpack_build("test-preview-webpack");
    fixture.create_wrangler_toml(wrangler_toml);

    preview_succeeds(&fixture);
}

#[test]
fn it_can_preview_rust_project() {
    let fixture = Fixture::new();

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

    let wrangler_toml = WranglerToml::rust("test-preview-rust");
    fixture.create_wrangler_toml(wrangler_toml);

    preview_succeeds(&fixture);
}

fn preview_succeeds_with(fixture: &Fixture, env: Option<&str>, expected: &str) {
    let _lock = fixture.lock();
    env::remove_var("CF_ACCOUNT_ID");
    env::remove_var("CF_ZONE_ID");
    let mut preview = Command::cargo_bin(env!("CARGO_PKG_NAME")).unwrap();
    preview.current_dir(fixture.get_path());
    preview.arg("preview").arg("--headless");
    if let Some(env) = env {
        preview.arg("--env").arg(env);
    }
    preview
        .assert()
        .stdout(predicates::str::contains(expected))
        .success();
}

fn preview_succeeds(fixture: &Fixture) {
    let _lock = fixture.lock();
    env::remove_var("CF_ACCOUNT_ID");
    let mut preview = Command::cargo_bin(env!("CARGO_PKG_NAME")).unwrap();
    preview.current_dir(fixture.get_path());
    preview.arg("preview").arg("--headless").assert().success();
}
