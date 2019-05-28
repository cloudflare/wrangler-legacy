use assert_cmd::prelude::*;

use std::fs;
use std::path::Path;
use std::process::Command;

#[test]
fn it_works() {
    let name = "init1";
    generate(Some(name));

    let wranglertoml_path = format!("{}/wrangler.toml", name);
    assert_eq!(Path::new(&wranglertoml_path).exists(), true);
    fs::remove_file(&wranglertoml_path).unwrap();

    init().current_dir(Path::new(name)).assert().success();

    cleanup(name);
}

#[test]
fn init_fails_if_wrangler_toml_exists() {
    let name = "init2";
    generate(Some(name));

    let wranglertoml_path = format!("{}/wrangler.toml", name);
    assert_eq!(Path::new(&wranglertoml_path).exists(), true);

    init().current_dir(Path::new(name)).assert().failure();

    cleanup(name);
}

fn init() -> Command {
    let mut wrangler = Command::cargo_bin(env!("CARGO_PKG_NAME")).unwrap();
    wrangler.arg("init");
    wrangler
}

fn generate(name: Option<&str>) {
    let mut wrangler = Command::cargo_bin(env!("CARGO_PKG_NAME")).unwrap();
    wrangler
        .arg("generate")
        .arg(name.unwrap())
        .assert()
        .success();
}

fn cleanup(name: &str) {
    fs::remove_dir_all(name).unwrap();
    assert_eq!(Path::new(name).exists(), false);
}
