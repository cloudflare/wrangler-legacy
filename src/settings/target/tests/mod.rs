use super::*;

use std::env;
use std::path::{Path, PathBuf};

#[test]
fn it_builds_from_config() {
    let toml_path = toml_fixture_path("default");

    let manifest = Manifest::new(&toml_path).unwrap();

    let target = manifest.get_target(None, false).unwrap();
    assert!(target.kv_namespaces.is_none());
}

#[test]
fn it_builds_from_environments_config() {
    let toml_path = toml_fixture_path("environments");
    let manifest = Manifest::new(&toml_path).unwrap();

    let target = manifest.get_target(None, false).unwrap();
    assert!(target.kv_namespaces.is_none());

    let target = manifest.get_target(Some("production"), false).unwrap();
    assert!(target.kv_namespaces.is_none());
}

#[test]
fn it_builds_from_environments_config_with_kv() {
    let toml_path = toml_fixture_path("kv_namespaces");

    let manifest = Manifest::new(&toml_path).unwrap();

    let target = manifest.get_target(None, false).unwrap();
    assert!(target.kv_namespaces.is_none());

    let target = manifest.get_target(Some("production"), false).unwrap();
    let kv_1 = KvNamespace {
        id: "somecrazylongidentifierstring".to_string(),
        binding: "prodKV-1".to_string(),
        bucket: None,
    };
    let kv_2 = KvNamespace {
        id: "anotherwaytoolongidstring".to_string(),
        binding: "prodKV-2".to_string(),
        bucket: None,
    };

    match target.kv_namespaces {
        Some(kv_namespaces) => {
            assert!(kv_namespaces.len() == 2);
            assert!(kv_namespaces.contains(&kv_1));
            assert!(kv_namespaces.contains(&kv_2));
        }
        None => assert!(false),
    }

    let target = manifest.get_target(Some("staging"), false).unwrap();
    let kv_1 = KvNamespace {
        id: "somecrazylongidentifierstring".to_string(),
        binding: "stagingKV-1".to_string(),
        bucket: None,
    };
    let kv_2 = KvNamespace {
        id: "anotherwaytoolongidstring".to_string(),
        binding: "stagingKV-2".to_string(),
        bucket: None,
    };
    match target.kv_namespaces {
        Some(kv_namespaces) => {
            assert!(kv_namespaces.len() == 2);
            assert!(kv_namespaces.contains(&kv_1));
            assert!(kv_namespaces.contains(&kv_2));
        }
        None => assert!(false),
    }
}

#[test]
fn it_builds_from_legacy_config() {
    let toml_path = legacy_toml_fixture_path("default");

    let manifest = Manifest::new(&toml_path).unwrap();
    let target = manifest.get_target(None, false).unwrap();

    assert!(target.kv_namespaces.is_none());
}

#[test]
fn it_builds_from_legacy_config_with_kv() {
    let toml_path = legacy_toml_fixture_path("kv_namespaces");

    let manifest = Manifest::new(&toml_path).unwrap();
    let target = manifest.get_target(None, false).unwrap();

    let kv_1 = KvNamespace {
        id: "somecrazylongidentifierstring".to_string(),
        binding: "prodKV".to_string(),
        bucket: None,
    };
    let kv_2 = KvNamespace {
        id: "anotherwaytoolongidstring".to_string(),
        binding: "stagingKV".to_string(),
        bucket: None,
    };

    match target.kv_namespaces {
        Some(kv_namespaces) => {
            assert!(kv_namespaces.len() == 2);
            assert!(kv_namespaces.contains(&kv_1));
            assert!(kv_namespaces.contains(&kv_2));
        }
        None => assert!(false),
    }
}

fn base_fixture_path() -> PathBuf {
    let current_dir = env::current_dir().unwrap();

    Path::new(&current_dir)
        .join("src")
        .join("settings")
        .join("target")
        .join("tests")
        .join("tomls")
}

fn legacy_toml_fixture_path(fixture: &str) -> PathBuf {
    base_fixture_path().join("legacy").join(fixture)
}

fn toml_fixture_path(fixture: &str) -> PathBuf {
    base_fixture_path().join(fixture)
}
