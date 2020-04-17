use wrangler::settings::toml::Manifest;

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
