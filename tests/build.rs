use std::fs;
use std::process::Command;
use std::str;

use assert_cmd::prelude::*;
mod fixtures;
use fixtures::{Fixture, WranglerToml};

#[test]
fn it_builds_webpack() {
    let fixture = Fixture::new();
    fixture.scaffold_webpack();

    let wrangler_toml = WranglerToml::webpack_build("test-build-webpack");
    fixture.create_wrangler_toml(wrangler_toml);

    build_creates_assets(&fixture, vec!["script.js"]);
}

#[test]
fn it_builds_webpack_site() {
    let fixture = Fixture::new_site();

    let wrangler_toml = WranglerToml::site("test-build-site");
    fixture.create_wrangler_toml(wrangler_toml);

    build_creates_assets(&fixture, vec!["script.js"]);
}

#[test]
fn it_builds_webpack_site_with_custom_webpack() {
    let fixture = Fixture::new_site();

    fixture.create_file(
        "workers-site/webpack.worker.js",
        r#"
        module.exports = { entry: "./workers-site/index.js" };
    "#,
    );

    let mut wrangler_toml = WranglerToml::site("test-build-site-specify-config");
    wrangler_toml.webpack_config = Some("workers-site/webpack.worker.js");
    fixture.create_wrangler_toml(wrangler_toml);

    build_creates_assets(&fixture, vec!["script.js"]);
}

#[test]
fn it_builds_with_webpack_single_js() {
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

    let wrangler_toml = WranglerToml::webpack_build("test-build-webpack-single-js");
    fixture.create_wrangler_toml(wrangler_toml);

    build_creates_assets(&fixture, vec!["script.js"]);
}

#[test]
fn it_builds_with_webpack_function_config_js() {
    let fixture = Fixture::new();
    fixture.scaffold_webpack();

    fixture.create_file(
        "webpack.config.js",
        r#"
        module.exports = () => ({ entry: "./index.js" });
    "#,
    );

    let wrangler_toml = WranglerToml::webpack_std_config("test-build-webpack-function");
    fixture.create_wrangler_toml(wrangler_toml);

    build_creates_assets(&fixture, vec!["script.js"]);
}

#[test]
fn it_builds_with_webpack_promise_config_js() {
    let fixture = Fixture::new();
    fixture.scaffold_webpack();

    fixture.create_file(
        "webpack.config.js",
        r#"
        module.exports = Promise.resolve({ entry: "./index.js" });
    "#,
    );

    let wrangler_toml = WranglerToml::webpack_std_config("test-build-webpack-promise");
    fixture.create_wrangler_toml(wrangler_toml);

    build_creates_assets(&fixture, vec!["script.js"]);
}

#[test]
fn it_builds_with_webpack_function_promise_config_js() {
    let fixture = Fixture::new();
    fixture.scaffold_webpack();

    fixture.create_file(
        "webpack.config.js",
        r#"
        module.exports = Promise.resolve({ entry: "./index.js" });
    "#,
    );

    let wrangler_toml = WranglerToml::webpack_std_config("test-build-webpack-function-promise");
    fixture.create_wrangler_toml(wrangler_toml);

    build_creates_assets(&fixture, vec!["script.js"]);
}

#[test]
fn it_builds_with_webpack_specify_config() {
    let fixture = Fixture::new();
    fixture.scaffold_webpack();

    fixture.create_file(
        "webpack.worker.js",
        r#"
        module.exports = { entry: "./index.js" };
    "#,
    );

    let wrangler_toml = WranglerToml::webpack_custom_config(
        "test-build-webpack-specify-config",
        "webpack.worker.js",
    );
    fixture.create_wrangler_toml(wrangler_toml);

    build_creates_assets(&fixture, vec!["script.js"]);
}

#[test]
fn it_builds_with_webpack_single_js_missing_package_main() {
    let fixture = Fixture::new();
    fixture.create_empty_js();

    fixture.create_file(
        "package.json",
        r#"
        {
            "name": "webpack_single_js_missing_package_main"
        }
    "#,
    );

    let wrangler_toml =
        WranglerToml::webpack_build("test-build-webpack-single-js-missing-package-main");
    fixture.create_wrangler_toml(wrangler_toml);

    build_fails_with(
        &fixture,
        "The `main` key in your `package.json` file is required",
    );
}

#[test]
fn it_fails_with_multiple_webpack_configs() {
    let fixture = Fixture::new();
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

    let wrangler_toml = WranglerToml::webpack_std_config("test-build-multiple-webpack-configs");
    fixture.create_wrangler_toml(wrangler_toml);

    build_fails_with(&fixture, "Multiple webpack configurations are not supported. You can specify a different path for your webpack configuration file in wrangler.toml with the `webpack_config` field");
}

#[test]
fn it_builds_with_webpack_wast() {
    let fixture = Fixture::new();
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

    let wrangler_toml = WranglerToml::webpack_std_config("test-build-webpack-wast");
    fixture.create_wrangler_toml(wrangler_toml);

    build_creates_assets(&fixture, vec!["script.js", "module.wasm"]);
}

#[test]
fn it_fails_with_webpack_target_node() {
    let fixture = Fixture::new();
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

    let wrangler_toml = WranglerToml::webpack_std_config("test-build-fails-webpack-target-node");
    fixture.create_wrangler_toml(wrangler_toml);

    build_fails_with(
        &fixture,
        "Building a Cloudflare Worker with target \"node\" is not supported",
    );
}

#[test]
fn it_fails_with_webpack_target_web() {
    let fixture = Fixture::new();
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

    let wrangler_toml = WranglerToml::webpack_std_config("test-build-fails-webpack-target-web");
    fixture.create_wrangler_toml(wrangler_toml);

    build_fails_with(
        &fixture,
        "Building a Cloudflare Worker with target \"web\" is not supported",
    );
}

#[test]
fn it_builds_with_webpack_target_webworker() {
    let fixture = Fixture::new();
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

    let wrangler_toml = WranglerToml::webpack_std_config("test-build-webpack-target-webworker");
    fixture.create_wrangler_toml(wrangler_toml);

    build_creates_assets(&fixture, vec!["script.js"]);
}

#[test]
fn it_builds_with_webpack_name_output() {
    let fixture = Fixture::new();
    fixture.scaffold_webpack();

    fixture.create_file(
        "webpack.config.js",
        r#"
        module.exports = {
            "entry": "./index.js",
            "devtool": "cheap-module-source-map"
        }
    "#,
    );
    fixture.create_file("index.js", "");

    let wrangler_toml = WranglerToml::webpack_std_config("test-build-webpack-name-output");
    fixture.create_wrangler_toml(wrangler_toml);

    build_creates_assets(&fixture, vec!["script.js"]);

    let out = fs::read_to_string(fixture.get_output_path().join("script.js")).unwrap();
    assert!(out.contains(r#"//# sourceMappingURL=worker.js.map"#));
}

#[test]
fn it_builds_with_webpack_name_output_warn() {
    let fixture = Fixture::new();
    fixture.scaffold_webpack();

    fixture.create_file(
        "webpack.config.js",
        r#"
        module.exports = {
            "entry": "./index.js",
            "output": {
                "filename": "hi.js"
            }
        }
    "#,
    );
    fixture.create_file("index.js", "");

    let wrangler_toml = WranglerToml::webpack_std_config("test-build-webpack-name-output-warn");
    fixture.create_wrangler_toml(wrangler_toml);

    let (_stdout, stderr) = build_creates_assets(&fixture, vec!["script.js"]);

    assert!(
        stderr.contains("webpack's output filename is being renamed"),
        "given: {}",
        stderr
    );
}

fn build_creates_assets_with_arg(
    fixture: &Fixture,
    script_names: Vec<&str>,
    args: Vec<&str>,
) -> (String, String) {
    let mut build = Command::cargo_bin(env!("CARGO_PKG_NAME")).unwrap();
    build.current_dir(fixture.get_path());
    build.arg("build");
    build.args(args);

    let output = build.output().expect("failed to execute process");
    assert!(output.status.success(), "Build failed: {:?}", output);

    for script_name in script_names {
        assert!(fixture.get_output_path().join(script_name).exists());
    }

    (
        str::from_utf8(&output.stdout).unwrap().to_string(),
        str::from_utf8(&output.stderr).unwrap().to_string(),
    )
}

fn build_creates_assets(fixture: &Fixture, script_names: Vec<&str>) -> (String, String) {
    let mut build = Command::cargo_bin(env!("CARGO_PKG_NAME")).unwrap();
    build.current_dir(fixture.get_path());
    build.arg("build");

    let output = build.output().expect("failed to execute process");
    assert!(output.status.success());

    for script_name in script_names {
        assert!(fixture.get_output_path().join(script_name).exists());
    }

    (
        str::from_utf8(&output.stdout).unwrap().to_string(),
        str::from_utf8(&output.stderr).unwrap().to_string(),
    )
}

fn build_fails_with(fixture: &Fixture, expected_message: &str) {
    let mut build = Command::cargo_bin(env!("CARGO_PKG_NAME")).unwrap();
    build.current_dir(fixture.get_path());
    build.arg("build");

    let output = build.output().expect("failed to execute process");
    assert!(!output.status.success());
    assert!(
        str::from_utf8(&output.stderr)
            .unwrap()
            .contains(expected_message),
        "expected {:?} not found, given: {:?}",
        expected_message,
        str::from_utf8(&output.stderr)
    );
}

#[test]
fn it_builds_with_webpack_target_webworker_with_custom_file() {
    let fixture = Fixture::new();
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

    let wrangler_toml = WranglerToml::webpack_std_config("test-build-webpack-target-webworker");
    let file_name = "wrangler-custom.toml";
    fixture.create_file(file_name, &toml::to_string(&wrangler_toml).unwrap());

    build_creates_assets_with_arg(&fixture, vec!["script.js"], vec!["-c", file_name]);
}
