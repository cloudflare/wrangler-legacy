use assert_cmd::prelude::*;
use std::env;
use std::process::Command;
use wrangler::fixtures::{EnvConfig, Fixture, KvConfig, WranglerToml, TEST_ENV_NAME};

#[test]
fn can_create_preview_namespace() {
    let fixture = Fixture::new();
    let mut wrangler_toml = WranglerToml::javascript("test-preview-javascript");
    wrangler_toml.kv_namespaces = Some(vec![KvConfig {
        binding: Some("BINDING"),
        id: Some("1234"),
    }]);
    fixture.create_wrangler_toml(wrangler_toml);
    preview_namespace_creation_succeeds(&fixture, None);
}

#[test]
fn can_create_env_preview_namespace() {
    let script_name = "test-env-preview-javascript";
    let fixture = Fixture::new();
    let mut env_config = EnvConfig::default();
    env_config.kv_namespaces = Some(vec![KvConfig {
        binding: Some("BINDING"),
        id: Some("1234"),
    }]);
    let wrangler_toml = WranglerToml::with_env(script_name, env_config);
    fixture.create_wrangler_toml(wrangler_toml);
    preview_namespace_creation_succeeds(&fixture, Some(TEST_ENV_NAME));
}

fn preview_namespace_creation_succeeds(fixture: &Fixture, env: Option<&str>) {
    env::remove_var("CF_ACCOUNT_ID");
    env::remove_var("CF_ZONE_ID");
    let mut create_namespace = Command::cargo_bin(env!("CARGO_PKG_NAME")).unwrap();
    create_namespace.current_dir(fixture.get_path());
    create_namespace
        .arg("kv:namespace")
        .arg("create")
        .arg("BINDING")
        .arg("--preview");
    if let Some(env) = env {
        create_namespace.arg("--env").arg(env);
    }
    create_namespace.assert().success();
}
