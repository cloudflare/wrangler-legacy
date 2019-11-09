#[macro_use]
extern crate lazy_static;

pub mod fixture;

use std::env;
use std::process::Command;
use std::sync::Mutex;

use assert_cmd::prelude::*;
use fixture::Fixture;

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
