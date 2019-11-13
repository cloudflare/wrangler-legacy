#[macro_use]
extern crate lazy_static;

pub mod fixture;

use std::process::Command;
use std::str;
use std::sync::Mutex;

use assert_cmd::prelude::*;
use fixture::{rust, Fixture};

lazy_static! {
    static ref BUILD_LOCK: Mutex<u8> = Mutex::new(0);
}

#[test]
fn it_builds_webpack() {
    let fixture = Fixture::new("webpack");
    fixture.scaffold_webpack();
    fixture.create_wrangler_toml(
        r#"
        type = "webpack"
    "#,
    );
    build_creates_assets(fixture, vec!["script.js"]);
}

#[test]
fn it_builds_with_webpack_single_js() {
    let fixture = Fixture::new("webpack_simple_js");
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
        type = "webpack"
    "#,
    );

    build_creates_assets(fixture, vec!["script.js"]);
}

#[test]
fn it_builds_with_webpack_function_config_js() {
    let fixture = Fixture::new("webpack_function_config_js");
    fixture.scaffold_webpack();

    fixture.create_file(
        "webpack.config.js",
        r#"
        module.exports = () => ({ entry: "./index.js" });
    "#,
    );

    fixture.create_wrangler_toml(
        r#"
        type = "webpack"
        webpack_config = "webpack.config.js"
    "#,
    );

    build_creates_assets(fixture, vec!["script.js"]);
}

#[test]
fn it_builds_with_webpack_promise_config_js() {
    let fixture = Fixture::new("webpack_promise_config_js");
    fixture.scaffold_webpack();

    fixture.create_file(
        "webpack.config.js",
        r#"
        module.exports = Promise.resolve({ entry: "./index.js" });
    "#,
    );

    fixture.create_wrangler_toml(
        r#"
        type = "webpack"
        webpack_config = "webpack.config.js"
    "#,
    );

    build_creates_assets(fixture, vec!["script.js"]);
}

#[test]
fn it_builds_with_webpack_function_promise_config_js() {
    let fixture = Fixture::new("webpack_function_promise_config_js");
    fixture.scaffold_webpack();

    fixture.create_file(
        "webpack.config.js",
        r#"
        module.exports = Promise.resolve({ entry: "./index.js" });
    "#,
    );

    fixture.create_wrangler_toml(
        r#"
        type = "webpack"
        webpack_config = "webpack.config.js"
    "#,
    );

    build_creates_assets(fixture, vec!["script.js"]);
}

#[test]
fn it_builds_with_webpack_specify_config() {
    let fixture = Fixture::new("webpack_specify_config");
    fixture.scaffold_webpack();

    fixture.create_file(
        "webpack.worker.js",
        r#"
        module.exports = { entry: "./index.js" };
    "#,
    );

    fixture.create_wrangler_toml(
        r#"
        type = "webpack"
        webpack_config = "webpack.worker.js"
    "#,
    );

    build_creates_assets(fixture, vec!["script.js"]);
}

#[test]
fn it_builds_with_webpack_single_js_missing_package_main() {
    let fixture = Fixture::new("webpack_single_js_missing_package_main");
    fixture.create_empty_js();

    fixture.create_file(
        "package.json",
        r#"
        {
            "name": "webpack_single_js_missing_package_main"
        }
    "#,
    );

    fixture.create_wrangler_toml(
        r#"
        type = "webpack"
    "#,
    );

    build_fails_with(
        &fixture,
        "The `main` key in your `package.json` file is required",
    );
    fixture.cleanup();
}

#[test]
fn it_fails_with_multiple_webpack_configs() {
    let fixture = Fixture::new("webpack_multiple_config");
    fixture.scaffold_webpack();

    fixture.create_file(
        "webpack.config.js",
        r#"
        module.exports = [
            { entry: "./a.js" },
            { entry: "./b.js" }
        ]
    "#,
    );

    fixture.create_wrangler_toml(
        r#"
        type = "webpack"
        webpack_config = "webpack.config.js"
    "#,
    );

    build_fails_with(&fixture, "Multiple webpack configurations are not supported. You can specify a different path for your webpack configuration file in wrangler.toml with the `webpack_config` field");
    fixture.cleanup();
}

#[test]
fn it_builds_with_webpack_wast() {
    let fixture = Fixture::new("webpack_wast");
    fixture.create_file(
        "package.json",
        r#"
        {
            "dependencies": {
                "wast-loader": "^1.8.5"
            }
        }
    "#,
    );

    fixture.create_file(
        "index.js",
        r#"
        (async function() {
            await import("./module.wast");
        })()
    "#,
    );

    fixture.create_file(
        "webpack.config.js",
        r#"
        module.exports = {
            entry: "./index.js",
            module: {
                rules: [
                    {
                        test: /\.wast$/,
                        loader: "wast-loader",
                        type: "webassembly/experimental"
                    }
                ]
            },
        }
    "#,
    );

    fixture.create_file("module.wast", "(module)");

    fixture.create_wrangler_toml(
        r#"
        type = "webpack"
        webpack_config = "webpack.config.js"
    "#,
    );

    build_creates_assets(fixture, vec!["script.js", "module.wasm"]);
}

#[test]
fn it_fails_with_webpack_target_node() {
    let fixture = Fixture::new("webpack_target_node");
    fixture.scaffold_webpack();

    fixture.create_file(
        "webpack.config.js",
        r#"
        module.exports = {
            "entry": "./index.js",
            "target": "node"
        }
    "#,
    );

    fixture.create_wrangler_toml(
        r#"
        type = "webpack"
        webpack_config = "webpack.config.js"
    "#,
    );

    build_fails_with(
        &fixture,
        "Building a Cloudflare Worker with target \"node\" is not supported",
    );
    fixture.cleanup();
}

#[test]
fn it_fails_with_webpack_target_web() {
    let fixture = Fixture::new("webpack_target_web");
    fixture.scaffold_webpack();

    fixture.create_file(
        "webpack.config.js",
        r#"
        module.exports = {
            "entry": "./index.js",
            "target": "web"
        }
    "#,
    );

    fixture.create_wrangler_toml(
        r#"
        type = "webpack"
        webpack_config = "webpack.config.js"
    "#,
    );

    build_fails_with(
        &fixture,
        "Building a Cloudflare Worker with target \"web\" is not supported",
    );
    fixture.cleanup();
}

#[test]
fn it_builds_with_webpack_target_webworker() {
    let fixture = Fixture::new("webpack_target_webworker");
    fixture.scaffold_webpack();

    fixture.create_file(
        "webpack.config.js",
        r#"
        module.exports = {
            "entry": "./index.js",
            "target": "webworker"
        }
    "#,
    );

    fixture.create_wrangler_toml(
        r#"
        type = "webpack"
        webpack_config = "webpack.config.js"
    "#,
    );

    build_creates_assets(fixture, vec!["script.js"]);
}

#[test]
fn it_builds_with_webpack_wasm_pack() {
    let fixture = Fixture::new("webpack_wasm_pack");

    fixture.create_dir("crate");
    fixture.create_file("crate/Cargo.toml", &rust::get_cargo_toml());

    fixture.create_dir("crate/src");
    fixture.create_file("crate/src/lib.rs", &rust::get_lib());
    fixture.create_file("crate/src/utils.rs", &rust::get_utils());

    fixture.create_wrangler_toml(
        r#"
        type = "webpack"
        webpack_config = "webpack.config.js"
    "#,
    );

    fixture.create_file(
        "webpack.config.js",
        r#"
        const path = require("path");
        const WasmPackPlugin = require("@wasm-tool/wasm-pack-plugin");
        
        module.exports = {
            "entry": "./index.js",
            "target": "webworker",
            plugins: [
                new WasmPackPlugin({
                    crateDirectory: path.resolve(__dirname, "crate"),
                }),
            ]
        }
    "#,
    );

    fixture.create_file(
        "package.json",
        r#"
        {
            "name": "webpack_wasm_pack",
            "main": "./index.js",
            "dependencies": {
            "@wasm-tool/wasm-pack-plugin": "^1.0.1"
            }
        }
    "#,
    );

    fixture.create_file(
        "index.js",
        r#"
        import("./crate/pkg/index.js").then(module => {
            module.greet();
        });
        "#,
    );

    build_creates_assets(fixture, vec!["script.js", "module.wasm"]);
}

fn build_creates_assets(fixture: Fixture, script_names: Vec<&str>) {
    // Lock to avoid having concurrent builds
    let _g = BUILD_LOCK.lock().unwrap();

    let mut build = Command::cargo_bin(env!("CARGO_PKG_NAME")).unwrap();
    build.current_dir(fixture.get_path());
    build.arg("build").assert().success();
    for script_name in script_names {
        assert!(fixture.get_output_path().join(script_name).exists());
    }
    fixture.cleanup();
}

fn build_fails_with(fixture: &Fixture, expected_message: &str) {
    // Lock to avoid having concurrent builds
    let _g = BUILD_LOCK.lock().unwrap();

    let mut build = Command::cargo_bin(env!("CARGO_PKG_NAME")).unwrap();
    build.current_dir(fixture.get_path());
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
