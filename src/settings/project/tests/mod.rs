use super::*;

use std::env;
use std::path::{Path, PathBuf};

#[test]
fn it_builds_from_config() {
    let toml_path = toml_fixture_path("default");

    let project = get_project_config(toml_path).unwrap();

    assert!(project.kv_namespaces.is_none());
}

#[test]
fn it_builds_from_config_with_kv() {
    let toml_path = toml_fixture_path("kv_namespaces");

    let project = get_project_config(toml_path).unwrap();

    let kv_1 = KvNamespace {
        id: "somecrazylongidentifierstring".to_string(),
        binding: "prodKV".to_string(),
    };
    let kv_2 = KvNamespace {
        id: "anotherwaytoolongidstring".to_string(),
        binding: "stagingKV".to_string(),
    };

    match project.kv_namespaces {
        Some(kv_namespaces) => {
            assert!(kv_namespaces.len() == 2);
            assert!(kv_namespaces.contains(&kv_1));
            assert!(kv_namespaces.contains(&kv_2));
        }
        None => assert!(false),
    }
}

fn toml_fixture_path(fixture: &str) -> PathBuf {
    let current_dir = env::current_dir().unwrap();

    // TODO: This is kind of stupid but idk worth it for now?
    Path::new(&current_dir)
        .join("src")
        .join("settings")
        .join("project")
        .join("tests")
        .join("tomls")
        .join(fixture)
}
