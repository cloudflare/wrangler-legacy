use wrangler::settings::toml::Manifest;
use wrangler::settings::toml::Service;

use std::str::FromStr;

#[test]
fn it_aliases_kv_namespaces() {
    let underscore_namespace_manifest = Manifest::from_str(
        r#"
    name = "worker"
    type = "javascript"
    workers_dev = true
    kv_namespaces = [
      { binding = "MY_KV", id = "1234" }
    ]
  "#,
    );

    let dash_namespace_manifest = Manifest::from_str(
        r#"
    name = "worker"
    type = "javascript"
    workers_dev = true
    kv-namespaces = [
      { binding = "MY_KV", id = "1234" }
    ]
  "#,
    );

    assert_eq!(underscore_namespace_manifest, dash_namespace_manifest);
}

#[test]
fn it_parses_services() {
    let manifest = Manifest::from_str(
        r#"
  name = "worker"
  type = "javascript"
  workers_dev = true
  experimental_services = [
    { name = "FOO", service = "foo", environment = "production" },
    { binding = "BAR", service = "bar", environment = "staging" }
  ]
  "#,
    )
    .unwrap();

    let expected_services = vec![
        Service {
            binding: "FOO".to_owned(),
            service: "foo".to_owned(),
            environment: "production".to_owned(),
        },
        Service {
            binding: "BAR".to_owned(),
            service: "bar".to_owned(),
            environment: "staging".to_owned(),
        },
    ];

    assert_eq!(manifest.experimental_services, Some(expected_services));
}

#[test]
fn it_parses_services_within_an_environment() {
    let manifest = Manifest::from_str(
        r#"
  name = "worker"
  type = "javascript"
  workers_dev = true
  [env.environment_name]
  experimental_services = [
    { name = "FOO", service = "foo", environment = "production" },
    { binding = "BAR", service = "bar", environment = "staging" }
  ]
  "#,
    )
    .unwrap();

    let environment = manifest
        .get_environment(Some("environment_name"))
        .unwrap()
        .unwrap();

    let expected_services = vec![
        Service {
            binding: "FOO".to_owned(),
            service: "foo".to_owned(),
            environment: "production".to_owned(),
        },
        Service {
            binding: "BAR".to_owned(),
            service: "bar".to_owned(),
            environment: "staging".to_owned(),
        },
    ];

    assert_eq!(manifest.experimental_services, None);
    assert_eq!(environment.experimental_services, Some(expected_services));
}
