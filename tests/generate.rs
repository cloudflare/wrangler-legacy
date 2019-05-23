use assert_cmd::prelude::*;

use std::fs;
use std::path::Path;
use std::process::Command;

#[test]
fn it_generates_with_defaults() {
    let name = "worker";
    generate(None, None);

    assert_eq!(Path::new(name).exists(), true);

    let wranglertoml_path = format!("{}/wrangler.toml", name);
    assert_eq!(Path::new(&wranglertoml_path).exists(), true);
    cleanup(name);
}

#[test]
fn it_generates_with_arguments() {
    let name = "example";
    let template = "https://github.com/cloudflare/rustwasm-worker-template";
    generate(Some(name), Some(template));

    assert_eq!(Path::new(name).exists(), true);

    let wranglertoml_path = format!("{}/wrangler.toml", name);
    assert_eq!(Path::new(&wranglertoml_path).exists(), true);
    cleanup(name);
}

fn generate(name: Option<&str>, template: Option<&str>) {
    let mut wrangler = Command::cargo_bin(env!("CARGO_PKG_NAME")).unwrap();
    if name.is_none() && template.is_none() {
        wrangler.arg("generate").assert().success();
    } else if name.is_some() && template.is_some() {
        wrangler.arg("generate").arg(name.unwrap()).arg(template.unwrap()).assert().success();
    }
}

fn cleanup(name: &str) {
    fs::remove_dir_all(name).unwrap();
    assert_eq!(Path::new(name).exists(), false);
}
