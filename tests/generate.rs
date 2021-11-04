use assert_cmd::prelude::*;

use std::fs;
use std::path::Path;
use std::process::Command;

#[test]
fn it_generates_with_defaults() {
    let name = "worker";
    generate(None, None, None);

    assert_eq!(Path::new(name).exists(), true);

    let wranglertoml_path = format!("{}/wrangler.toml", name);
    assert_eq!(Path::new(&wranglertoml_path).exists(), true);
    cleanup(name);
}

#[test]
fn it_generates_with_arguments() {
    let name = "example";
    let template = "https://github.com/cloudflare/rustwasm-worker-template";
    let project_type = "webpack";
    generate(Some(name), Some(template), Some(project_type));

    assert_eq!(Path::new(name).exists(), true);

    let wranglertoml_path = format!("{}/wrangler.toml", name);
    assert_eq!(Path::new(&wranglertoml_path).exists(), true);
    let wranglertoml_text = fs::read_to_string(wranglertoml_path).unwrap();
    assert!(wranglertoml_text.contains(project_type));
    cleanup(name);
}

#[test]
fn it_generates_toml_file_in_correct_directory() {
    let name = "collision-example";
    let expected_name = "collision-example-1";
    let template = "https://github.com/cloudflare/rustwasm-worker-template";
    let project_type = "webpack";
    fs::create_dir_all(Path::new(name)).unwrap();

    generate(Some(name), Some(template), Some(project_type));
    assert_eq!(Path::new(expected_name).exists(), true);

    let wranglertoml_path = format!("{}/wrangler.toml", expected_name);
    assert_eq!(Path::new(&wranglertoml_path).exists(), true);

    let wranglertoml_text = fs::read_to_string(wranglertoml_path).unwrap();
    assert!(wranglertoml_text.contains(expected_name));

    let unexpected_wranglertoml_path = format!("{}/wrangler.toml", name);
    assert_eq!(Path::new(&unexpected_wranglertoml_path).exists(), false);

    cleanup(name);
    cleanup(expected_name);
}

pub fn generate(name: Option<&str>, template: Option<&str>, project_type: Option<&str>) {
    let mut wrangler = Command::cargo_bin(env!("CARGO_PKG_NAME")).unwrap();
    if name.is_none() && template.is_none() && project_type.is_none() {
        wrangler.arg("generate").assert().success();
    } else if name.is_some() && template.is_some() && project_type.is_some() {
        wrangler
            .arg("generate")
            .arg(name.unwrap())
            .arg(template.unwrap())
            .arg("--type")
            .arg(project_type.unwrap())
            .assert()
            .success();
    }
}

fn cleanup(name: &str) {
    fs::remove_dir_all(name).unwrap();
    assert_eq!(Path::new(name).exists(), false);
}
