mod deployments;

use super::*;

use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::str::FromStr;

#[path = "../../../../tests/fixtures/mod.rs"]
mod fixtures;
use fixtures::{EnvConfig, WranglerToml, TEST_ENV_NAME};

#[test]
fn it_builds_from_config() {
    let toml_path = toml_fixture_path("default");

    let manifest = Manifest::new(&toml_path).unwrap();

    let target = manifest.get_target(None, false).unwrap();
    assert!(target.kv_namespaces.is_empty());
}

#[test]
fn it_builds_from_environments_config() {
    let toml_path = toml_fixture_path("environments");
    let manifest = Manifest::new(&toml_path).unwrap();

    let target = manifest.get_target(None, false).unwrap();
    assert!(target.kv_namespaces.is_empty());

    let target = manifest.get_target(Some("production"), false).unwrap();
    assert!(target.kv_namespaces.is_empty());
}

#[test]
fn it_builds_from_environments_config_with_kv() {
    let toml_path = toml_fixture_path("kv_namespaces");

    let manifest = Manifest::new(&toml_path).unwrap();

    let target = manifest.get_target(None, false).unwrap();
    assert!(target.kv_namespaces.is_empty());

    let target = manifest.get_target(Some("production"), false).unwrap();
    let kv_1 = KvNamespace {
        id: "somecrazylongidentifierstring".to_string(),
        binding: "prodKV-1".to_string(),
    };
    let kv_2 = KvNamespace {
        id: "anotherwaytoolongidstring".to_string(),
        binding: "prodKV-2".to_string(),
    };

    if target.kv_namespaces.is_empty() {
        panic!("found no kv namespaces");
    } else {
        assert_eq!(target.kv_namespaces.len(), 2);
        assert!(target.kv_namespaces.contains(&kv_1));
        assert!(target.kv_namespaces.contains(&kv_2));
    }

    let target = manifest.get_target(Some("staging"), false).unwrap();
    let kv_1 = KvNamespace {
        id: "somecrazylongidentifierstring".to_string(),
        binding: "stagingKV-1".to_string(),
    };
    let kv_2 = KvNamespace {
        id: "anotherwaytoolongidstring".to_string(),
        binding: "stagingKV-2".to_string(),
    };
    if target.kv_namespaces.is_empty() {
        panic!("found no kv namespaces");
    } else {
        assert_eq!(target.kv_namespaces.len(), 2);
        assert!(target.kv_namespaces.contains(&kv_1));
        assert!(target.kv_namespaces.contains(&kv_2));
    }
}

#[test]
fn it_builds_from_environments_config_with_r2() {
    let toml_path = toml_fixture_path("r2_buckets");

    let manifest = Manifest::new(&toml_path).unwrap();

    let target = manifest.get_target(None, false).unwrap();
    assert!(target.r2_buckets.is_empty());

    let target = manifest.get_target(Some("production"), false).unwrap();
    let r2_1 = R2Bucket {
        bucket_name: "long_bucket_name".to_string(),
        binding: "prodr2_1".to_string(),
    };
    let r2_2 = R2Bucket {
        bucket_name: "another_long_bucket_name".to_string(),
        binding: "prodr2_2".to_string(),
    };

    if target.r2_buckets.is_empty() {
        panic!("found no r2 buckets");
    } else {
        assert_eq!(target.r2_buckets.len(), 2);
        assert!(target.r2_buckets.contains(&r2_1));
        assert!(target.r2_buckets.contains(&r2_2));
    }

    let target = manifest.get_target(Some("staging"), false).unwrap();
    let r2_1 = R2Bucket {
        bucket_name: "long_bucket_name".to_string(),
        binding: "stagingr2_1".to_string(),
    };
    let r2_2 = R2Bucket {
        bucket_name: "another_long_bucket_name".to_string(),
        binding: "stagingr2_2".to_string(),
    };
    if target.r2_buckets.is_empty() {
        panic!("found no r2 buckets");
    } else {
        assert_eq!(target.r2_buckets.len(), 2);
        assert!(target.r2_buckets.contains(&r2_1));
        assert!(target.r2_buckets.contains(&r2_2));
    }
}

#[test]
fn parses_same_from_config_path_as_string() {
    env::remove_var("CF_ACCOUNT_ID");
    env::remove_var("CF_ZONE_ID");
    let config_path = toml_fixture_path("environments.toml");
    let string_toml = fs::read_to_string(&config_path).unwrap();

    let manifest_from_string = Manifest::from_str(&string_toml).unwrap();
    let manifest_from_config = Manifest::new(&config_path).unwrap();

    assert_eq!(manifest_from_config, manifest_from_string);
}

#[test]
fn it_returns_top_level_name_when_no_env() {
    let top_level_name = "worker";

    let with_name_no_env = WranglerToml::webpack(top_level_name);
    let manifest = Manifest::from_str(&toml::to_string(&with_name_no_env).unwrap()).unwrap();

    assert_eq!(manifest.worker_name(None), top_level_name);
}

#[test]
fn it_concatenates_top_level_with_env_when_env_omits_name() {
    let top_level_name = "worker";

    let with_name_with_env = WranglerToml::with_env(top_level_name, EnvConfig::default());
    let manifest = Manifest::from_str(&toml::to_string(&with_name_with_env).unwrap()).unwrap();

    assert_eq!(
        manifest.worker_name(Some(TEST_ENV_NAME)),
        format!("{}-{}", top_level_name, TEST_ENV_NAME)
    );
}

#[test]
fn it_uses_env_name_when_provided() {
    let top_level_name = "worker";
    let custom_env_name = "george";

    let env_config = EnvConfig::custom_script_name(custom_env_name);
    let with_name_env_override = WranglerToml::with_env(top_level_name, env_config);
    let manifest = Manifest::from_str(&toml::to_string(&with_name_env_override).unwrap()).unwrap();

    assert_eq!(manifest.worker_name(Some(TEST_ENV_NAME)), custom_env_name);
}

fn base_fixture_path() -> PathBuf {
    let current_dir = env::current_dir().unwrap();

    Path::new(&current_dir)
        .join("src")
        .join("settings")
        .join("toml")
        .join("tests")
        .join("tomls")
}

fn toml_fixture_path(fixture: &str) -> PathBuf {
    base_fixture_path().join(fixture).with_extension("toml")
}
