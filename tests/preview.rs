#[macro_use]
extern crate lazy_static;

pub mod fixture;

use fixture::WranglerToml;

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

    let wrangler_toml = WranglerToml::webpack_no_config("test-preview-webpack");
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

fn preview_succeeds(fixture: &Fixture) {
    let _lock = fixture.lock();
    env::remove_var("CF_ACCOUNT_ID");
    let mut preview = Command::cargo_bin(env!("CARGO_PKG_NAME")).unwrap();
    preview.current_dir(fixture.get_path());
    preview.arg("preview").arg("--headless").assert().success();
}
