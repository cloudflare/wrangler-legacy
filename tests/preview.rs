#[macro_use]
extern crate lazy_static;

pub mod fixture;

use fixture::{EnvConfig, WranglerToml, TEST_ENV_NAME};

use std::collections::HashMap;
use std::env;
use std::process::Command;

use assert_cmd::prelude::*;
use fixture::Fixture;

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
    fixture.create_dir("src");
    fixture.create_dir("worker");

    fixture.create_file(
        "src/lib.rs",
        r#"
        extern crate cfg_if;
        extern crate wasm_bindgen;

        mod utils;

        use cfg_if::cfg_if;
        use wasm_bindgen::prelude::*;

        cfg_if! {
            // When the `wee_alloc` feature is enabled, use `wee_alloc` as the global
            // allocator.
            if #[cfg(feature = "wee_alloc")] {
                extern crate wee_alloc;
                #[global_allocator]
                static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;
            }
        }

        #[wasm_bindgen]
        pub fn greet() -> String {
            "Hello, wasm-worker!".to_string()
        }
    "#,
    );

    fixture.create_file(
        "src/utils.rs",
        r#"
        use cfg_if::cfg_if;

        cfg_if! {
            // When the `console_error_panic_hook` feature is enabled, we can call the
            // `set_panic_hook` function at least once during initialization, and then
            // we will get better error messages if our code ever panics.
            //
            // For more details see
            // https://github.com/rustwasm/console_error_panic_hook#readme
            if #[cfg(feature = "console_error_panic_hook")] {
                extern crate console_error_panic_hook;
                pub use self::console_error_panic_hook::set_once as set_panic_hook;
            } else {
                #[inline]
                pub fn set_panic_hook() {}
            }
        }
    "#,
    );

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

    fixture.create_file(
        "Cargo.toml",
        r#"
        [package]
        name = "worker"
        version = "0.1.0"
        authors = ["The Wrangler Team <wrangler@cloudflare.com>"]
        edition = "2018"

        [lib]
        crate-type = ["cdylib", "rlib"]

        [features]
        default = ["console_error_panic_hook"]

        [dependencies]
        cfg-if = "0.1.2"
        wasm-bindgen = "0.2"

        # The `console_error_panic_hook` crate provides better debugging of panics by
        # logging them with `console.error`. This is great for development, but requires
        # all the `std::fmt` and `std::panicking` infrastructure, so isn't great for
        # code size when deploying.
        console_error_panic_hook = { version = "0.1.1", optional = true }

        # `wee_alloc` is a tiny allocator for wasm that is only ~1K in code size
        # compared to the default allocator's ~10K. It is slower than the default
        # allocator, however.
        #
        # Unfortunately, `wee_alloc` requires nightly Rust when targeting wasm for now.
        wee_alloc = { version = "0.4.2", optional = true }

        [dev-dependencies]
        wasm-bindgen-test = "0.2"

        [profile.release]
        # Tell `rustc` to optimize for small code size.
        opt-level = "s"
    "#,
    );

    let wrangler_toml = WranglerToml::rust("test-preview-rust");
    fixture.create_wrangler_toml(wrangler_toml);

    preview_succeeds(&fixture);
}

#[test]
fn it_previews_with_config_text() {
    let fixture = Fixture::new();
    fixture.create_file(
        "index.js",
        r#"
        addEventListener('fetch', event => {
            event.respondWith(handleRequest(event.request))
        })
        
        async function handleRequest(request) {
            return new Response(CONFIG_TEST)
        }
    "#,
    );
    fixture.create_default_package_json();

    let test_value: &'static str = "sdhftiuyrtdhfjgpoopuyrdfjgkyitudrhf";

    let mut wrangler_toml = WranglerToml::javascript("test-preview-with-config");
    let mut config: HashMap<&'static str, &'static str> = HashMap::new();
    config.insert("CONFIG_TEST", test_value);
    wrangler_toml.vars = Some(config);
    fixture.create_wrangler_toml(wrangler_toml);

    preview_succeeds_with(&fixture, None, test_value);
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
