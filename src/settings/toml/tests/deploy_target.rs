use super::wrangler_toml::{EnvConfig, WranglerToml};

use crate::settings::toml::deploy_target::Zoned;
use crate::settings::toml::route::Route;
use crate::settings::toml::DeployTarget;
use crate::settings::toml::Manifest;

// TOP LEVEL TESTS
#[test]
fn it_can_get_a_top_level_zoneless_deploy_target() {
    let test_toml = WranglerToml::webpack_zoneless("zoneless", true);
    let toml_string = toml::to_string(&test_toml).unwrap();
    let manifest = Manifest::new_from_string(toml_string).unwrap();

    let environment = None;
    let actual_deploy_target = manifest.deploy_target(environment).unwrap();
    let expected_deploy_target = DeployTarget::Zoneless;

    assert_eq!(actual_deploy_target, expected_deploy_target);
}

#[test]
fn it_errors_on_zoneless_deploy_target_missing_workers_dev() {
    let test_toml = WranglerToml::webpack("zoneless_missing");
    let toml_string = toml::to_string(&test_toml).unwrap();
    let manifest = Manifest::new_from_string(toml_string).unwrap();

    let environment = None;

    assert!(manifest.deploy_target(environment).is_err());
}

#[test]
fn it_errors_on_deploy_target_missing_name() {
    let test_toml = WranglerToml::webpack_zoneless("", true);
    let toml_string = toml::to_string(&test_toml).unwrap();
    let manifest = Manifest::new_from_string(toml_string).unwrap();

    let environment = None;

    assert!(manifest.deploy_target(environment).is_err());
}

#[test]
fn it_errors_on_zoneless_deploy_target_workers_dev_false() {
    let test_toml = WranglerToml::webpack_zoneless("zoneless_false", false);
    let toml_string = toml::to_string(&test_toml).unwrap();
    let manifest = Manifest::new_from_string(toml_string).unwrap();

    let environment = None;

    assert!(manifest.deploy_target(environment).is_err());
}

#[test]
fn it_can_get_a_single_route_zoned_deploy_target() {
    let script = "single_route_zoned";
    let pattern = "hostname.tld/*";
    let zone_id = "samplezoneid";

    let test_toml = WranglerToml::webpack_zoned_single_route(script, zone_id, pattern);
    let toml_string = toml::to_string(&test_toml).unwrap();
    let manifest = Manifest::new_from_string(toml_string).unwrap();

    let environment = None;
    let actual_deploy_target = manifest.deploy_target(environment).unwrap();
    let expected_routes = vec![Route {
        script: Some(script.to_string()),
        pattern: pattern.to_string(),
        id: None,
    }];
    let expected_deploy_target = DeployTarget::Zoned(Zoned {
        zone_id: zone_id.to_string(),
        routes: expected_routes,
    });

    assert_eq!(actual_deploy_target, expected_deploy_target);
}

#[test]
fn it_can_get_a_single_route_zoned_deploy_target_workers_dev_false() {
    let script = "single_route_zoned_workers_dev_false";
    let pattern = "hostname.tld/*";
    let zone_id = "samplezoneid";

    let mut test_toml = WranglerToml::webpack_zoned_single_route(script, zone_id, pattern);
    test_toml.workers_dev = Some(false);
    let toml_string = toml::to_string(&test_toml).unwrap();
    let manifest = Manifest::new_from_string(toml_string).unwrap();

    let environment = None;
    let actual_deploy_target = manifest.deploy_target(environment).unwrap();
    let expected_routes = vec![Route {
        script: Some(script.to_string()),
        pattern: pattern.to_string(),
        id: None,
    }];
    let expected_deploy_target = DeployTarget::Zoned(Zoned {
        zone_id: zone_id.to_string(),
        routes: expected_routes,
    });

    assert_eq!(actual_deploy_target, expected_deploy_target);
}

#[test]
fn it_errors_on_single_route_deploy_target_empty_zone_id() {
    let script = "single_route_empty_zone_id";
    let pattern = "hostname.tld/*";
    let zone_id = "";

    let test_toml = WranglerToml::webpack_zoned_single_route(script, zone_id, pattern);
    let toml_string = toml::to_string(&test_toml).unwrap();
    let manifest = Manifest::new_from_string(toml_string).unwrap();

    let environment = None;

    assert!(manifest.deploy_target(environment).is_err());
}

#[test]
fn it_errors_on_single_route_deploy_target_missing_zone_id() {
    let script = "single_route_empty_zone_id";
    let pattern = "hostname.tld/*";

    let mut test_toml = WranglerToml::webpack_zoned_single_route(script, "", pattern);
    test_toml.zone_id = None;
    let toml_string = toml::to_string(&test_toml).unwrap();
    let manifest = Manifest::new_from_string(toml_string).unwrap();

    let environment = None;

    assert!(manifest.deploy_target(environment).is_err());
}

#[test]
fn it_errors_on_single_route_deploy_target_empty_route() {
    let script = "single_route_empty_route";
    let pattern = "";
    let zone_id = "samplezoneid";

    let test_toml = WranglerToml::webpack_zoned_single_route(script, zone_id, pattern);
    let toml_string = toml::to_string(&test_toml).unwrap();
    let manifest = Manifest::new_from_string(toml_string).unwrap();

    let environment = None;

    assert!(manifest.deploy_target(environment).is_err());
}

#[test]
fn it_errors_on_single_route_deploy_target_missing_route() {
    let script = "single_route_missing_route";
    let zone_id = "samplezoneid";

    let mut test_toml = WranglerToml::webpack_zoned_single_route(script, zone_id, "");
    test_toml.route = None;
    let toml_string = toml::to_string(&test_toml).unwrap();
    let manifest = Manifest::new_from_string(toml_string).unwrap();

    let environment = None;

    assert!(manifest.deploy_target(environment).is_err());
}

#[test]
fn it_can_get_a_multi_route_zoned_deploy_target() {
    let script = "multi_route_zoned";
    let patterns = ["hostname.tld/*", "blog.hostname.tld/*"];
    let zone_id = "samplezoneid";

    let mut test_toml = WranglerToml::webpack(script);
    test_toml.routes = Some(patterns.to_vec());
    test_toml.zone_id = Some(zone_id);
    let toml_string = toml::to_string(&test_toml).unwrap();
    let manifest = Manifest::new_from_string(toml_string).unwrap();

    let expected_routes = patterns
        .iter()
        .map(|p| Route {
            script: Some(script.to_string()),
            pattern: p.to_string(),
            id: None,
        })
        .collect();
    let expected_deploy_target = DeployTarget::Zoned(Zoned {
        zone_id: zone_id.to_string(),
        routes: expected_routes,
    });

    let environment = None;
    let actual_deploy_target = manifest.deploy_target(environment).unwrap();

    assert_eq!(actual_deploy_target, expected_deploy_target);
}

#[test]
fn it_can_get_a_multi_route_zoned_deploy_target_workers_dev_false() {
    let script = "multi_route_zoned_workers_dev_false";
    let patterns = ["hostname.tld/*", "blog.hostname.tld/*"];
    let zone_id = "samplezoneid";

    let mut test_toml = WranglerToml::webpack(script);
    test_toml.workers_dev = Some(false);
    test_toml.routes = Some(patterns.to_vec());
    test_toml.zone_id = Some(zone_id);
    let toml_string = toml::to_string(&test_toml).unwrap();
    let manifest = Manifest::new_from_string(toml_string).unwrap();

    let expected_routes = patterns
        .iter()
        .map(|p| Route {
            script: Some(script.to_string()),
            pattern: p.to_string(),
            id: None,
        })
        .collect();
    let expected_deploy_target = DeployTarget::Zoned(Zoned {
        zone_id: zone_id.to_string(),
        routes: expected_routes,
    });

    let environment = None;
    let actual_deploy_target = manifest.deploy_target(environment).unwrap();

    assert_eq!(actual_deploy_target, expected_deploy_target);
}

#[test]
fn it_errors_on_multi_route_deploy_target_empty_zone_id() {
    let script = "multi_route_empty_zone_id";
    let patterns = ["hostname.tld/*", "blog.hostname.tld/*"];
    let zone_id = "";

    let test_toml = WranglerToml::webpack_zoned_multi_route(script, zone_id, patterns.to_vec());
    let toml_string = toml::to_string(&test_toml).unwrap();
    let manifest = Manifest::new_from_string(toml_string).unwrap();

    let environment = None;

    assert!(manifest.deploy_target(environment).is_err());
}

#[test]
fn it_errors_on_multi_route_deploy_target_missing_zone_id() {
    let script = "multi_route_missing_zone_id";
    let patterns = ["hostname.tld/*", "blog.hostname.tld/*"];
    let zone_id = "";

    let mut test_toml = WranglerToml::webpack_zoned_multi_route(script, zone_id, patterns.to_vec());
    test_toml.zone_id = None;
    let toml_string = toml::to_string(&test_toml).unwrap();
    let manifest = Manifest::new_from_string(toml_string).unwrap();

    let environment = None;

    assert!(manifest.deploy_target(environment).is_err());
}

#[test]
fn it_errors_on_multi_route_deploy_target_empty_routes_list() {
    let script = "multi_route_empty_routes_list";
    let patterns = [];
    let zone_id = "samplezoneid";

    let test_toml = WranglerToml::webpack_zoned_multi_route(script, zone_id, patterns.to_vec());
    let toml_string = toml::to_string(&test_toml).unwrap();
    let manifest = Manifest::new_from_string(toml_string).unwrap();

    let environment = None;

    assert!(manifest.deploy_target(environment).is_err());
}

#[test]
fn it_errors_on_multi_route_deploy_target_empty_route() {
    let script = "multi_route_empty_route";
    let patterns = [""];
    let zone_id = "samplezoneid";

    let test_toml = WranglerToml::webpack_zoned_multi_route(script, zone_id, patterns.to_vec());
    let toml_string = toml::to_string(&test_toml).unwrap();
    let manifest = Manifest::new_from_string(toml_string).unwrap();

    let environment = None;

    assert!(manifest.deploy_target(environment).is_err());
}

#[test]
fn it_errors_on_deploy_target_route_and_routes() {
    let script = "route_and_routes";
    let pattern = "hostname.tld/*";
    let patterns = ["blog.hostname.tld/*"];
    let zone_id = "samplezoneid";

    let mut test_toml = WranglerToml::webpack_zoned_single_route(script, zone_id, pattern);
    test_toml.routes = Some(patterns.to_vec());
    let toml_string = toml::to_string(&test_toml).unwrap();
    let manifest = Manifest::new_from_string(toml_string).unwrap();

    let environment = None;

    assert!(manifest.deploy_target(environment).is_err());
}

#[test]
fn it_errors_on_deploy_target_route_and_workers_dev_true() {
    let script = "route_and_workers_dev";
    let pattern = "hostname.tld/*";
    let zone_id = "samplezoneid";

    let mut test_toml = WranglerToml::webpack_zoned_single_route(script, zone_id, pattern);
    test_toml.workers_dev = Some(true);
    let toml_string = toml::to_string(&test_toml).unwrap();
    let manifest = Manifest::new_from_string(toml_string).unwrap();

    let environment = None;

    assert!(manifest.deploy_target(environment).is_err());
}

#[test]
fn it_errors_on_deploy_target_routes_and_workers_dev_true() {
    let script = "routes_and_workers_dev";
    let patterns = ["blog.hostname.tld/*"];
    let zone_id = "samplezoneid";

    let mut test_toml = WranglerToml::webpack_zoned_multi_route(script, zone_id, patterns.to_vec());
    test_toml.workers_dev = Some(true);
    let toml_string = toml::to_string(&test_toml).unwrap();
    let manifest = Manifest::new_from_string(toml_string).unwrap();

    let environment = None;

    assert!(manifest.deploy_target(environment).is_err());
}

// ENVIRONMENT TESTS
// Top level empty
#[test]
fn when_top_level_empty_env_empty() {
    let script = "top_level_empty_env_empty";
    let env_name = "test";
    let env_config = EnvConfig::default();

    let test_toml = WranglerToml::webpack_with_env(script, env_name, env_config);
    let toml_string = toml::to_string(&test_toml).unwrap();
    let manifest = Manifest::new_from_string(toml_string).unwrap();

    let actual_deploy_target = manifest.deploy_target(Some(env_name));

    assert!(actual_deploy_target.is_err());

    // if env only includes zone id, error
    let mut env_config = EnvConfig::default();
    env_config.zone_id = Some("samplezoneid");

    let test_toml = WranglerToml::webpack_with_env(script, env_name, env_config);
    let toml_string = toml::to_string(&test_toml).unwrap();
    let manifest = Manifest::new_from_string(toml_string).unwrap();

    let actual_deploy_target = manifest.deploy_target(Some(env_name));

    assert!(actual_deploy_target.is_err());
}

#[test]
fn when_top_level_empty_zoneless_env() {
    let env_name = "test";

    // TODO: THIS TEST SHOULD PASS
    // when env.workers_dev = false
    let env_config = EnvConfig::zoneless(false);

    let script = "top_level_empty_env_empty";
    let test_toml = WranglerToml::webpack_with_env(script, env_name, env_config);
    let toml_string = toml::to_string(&test_toml).unwrap();
    let manifest = Manifest::new_from_string(toml_string).unwrap();

    let actual_deploy_target = manifest.deploy_target(Some(env_name));

    assert!(actual_deploy_target.is_err());

    // when env.workers_dev = true
    let env_config = EnvConfig::zoneless(true);

    let script = "top_level_empty_env_zoneless_true";
    let test_toml = WranglerToml::webpack_with_env(script, env_name, env_config);
    let toml_string = toml::to_string(&test_toml).unwrap();
    let manifest = Manifest::new_from_string(toml_string).unwrap();

    let actual_deploy_target = manifest.deploy_target(Some(env_name)).unwrap();

    assert_eq!(actual_deploy_target, DeployTarget::Zoneless);
}

#[test]
fn when_top_level_empty_zoned_single_route_env() {
    let env_name = "test";
    let zone_id = "samplezoneid";

    // when route is empty, error
    let pattern = "";
    let env_config = EnvConfig::zoned_single_route(zone_id, pattern);

    let script = "top_level_empty_env_zoned_single_route_empty";
    let test_toml = WranglerToml::webpack_with_env(script, env_name, env_config);
    let toml_string = toml::to_string(&test_toml).unwrap();
    let manifest = Manifest::new_from_string(toml_string).unwrap();

    let actual_deploy_target = manifest.deploy_target(Some(env_name));

    assert!(actual_deploy_target.is_err());

    // when zone id is missing, error
    let pattern = "hostname.tld/*";
    let env_config = EnvConfig::zoned_single_route("", pattern);

    let script = "top_level_empty_env_zoned_single_route_no_zone_id";
    let test_toml = WranglerToml::webpack_with_env(script, env_name, env_config);
    let toml_string = toml::to_string(&test_toml).unwrap();
    let manifest = Manifest::new_from_string(toml_string).unwrap();

    let actual_deploy_target = manifest.deploy_target(Some(env_name));

    assert!(actual_deploy_target.is_err());

    // when only zone id is present (no routes)
    let mut env_config = EnvConfig::default();
    env_config.zone_id = Some("samplezoneid");

    let test_toml = WranglerToml::webpack_with_env(script, env_name, env_config);
    let toml_string = toml::to_string(&test_toml).unwrap();
    let manifest = Manifest::new_from_string(toml_string).unwrap();

    let actual_deploy_target = manifest.deploy_target(Some(env_name));

    assert!(actual_deploy_target.is_err());

    // when zone id is present, all good
    let pattern = "hostname.tld/*";
    let env_config = EnvConfig::zoned_single_route(zone_id, pattern);

    let script = "top_level_empty_env_zoned_single_route_no_zone_id";
    let test_toml = WranglerToml::webpack_with_env(script, env_name, env_config);
    let toml_string = toml::to_string(&test_toml).unwrap();
    let manifest = Manifest::new_from_string(toml_string).unwrap();

    let actual_deploy_target = manifest.deploy_target(Some(env_name)).unwrap();

    let expected_name = manifest.worker_name(Some(env_name));

    let expected_routes = vec![Route {
        script: Some(expected_name),
        pattern: pattern.to_string(),
        id: None,
    }];
    let expected_deploy_target = DeployTarget::Zoned(Zoned {
        zone_id: zone_id.to_string(),
        routes: expected_routes,
    });

    assert_eq!(actual_deploy_target, expected_deploy_target);
}

#[test]
fn when_top_level_empty_zoned_multi_route_env() {
    let env_name = "test";
    let zone_id = "samplezoneid";

    // when routes list is empty, error
    let patterns = [];
    let env_config = EnvConfig::zoned_multi_route(zone_id, patterns.to_vec());

    let script = "top_level_empty_env_zoned_multi_route_empty_list";
    let test_toml = WranglerToml::webpack_with_env(script, env_name, env_config);
    let toml_string = toml::to_string(&test_toml).unwrap();
    let manifest = Manifest::new_from_string(toml_string).unwrap();

    let actual_deploy_target = manifest.deploy_target(Some(env_name));

    assert!(actual_deploy_target.is_err());

    // when route is empty, error
    let patterns = [""];
    let env_config = EnvConfig::zoned_multi_route(zone_id, patterns.to_vec());

    let script = "top_level_empty_env_zoned_multi_route_empty";
    let test_toml = WranglerToml::webpack_with_env(script, env_name, env_config);
    let toml_string = toml::to_string(&test_toml).unwrap();
    let manifest = Manifest::new_from_string(toml_string).unwrap();

    let actual_deploy_target = manifest.deploy_target(Some(env_name));

    assert!(actual_deploy_target.is_err());

    // when zone id is missing, error
    let patterns = ["hostname.tld/*"];
    let env_config = EnvConfig::zoned_multi_route("", patterns.to_vec());

    let script = "top_level_empty_env_zoned_multi_route_no_zone_id";
    let test_toml = WranglerToml::webpack_with_env(script, env_name, env_config);
    let toml_string = toml::to_string(&test_toml).unwrap();
    let manifest = Manifest::new_from_string(toml_string).unwrap();

    let actual_deploy_target = manifest.deploy_target(Some(env_name));

    assert!(actual_deploy_target.is_err());

    // when zone id is present, all good
    let patterns = ["hostname.tld/*"];
    let env_config = EnvConfig::zoned_multi_route(zone_id, patterns.to_vec());

    let script = "top_level_empty_env_zoned_multi_route_no_zone_id";
    let test_toml = WranglerToml::webpack_with_env(script, env_name, env_config);
    let toml_string = toml::to_string(&test_toml).unwrap();
    let manifest = Manifest::new_from_string(toml_string).unwrap();

    let actual_deploy_target = manifest.deploy_target(Some(env_name)).unwrap();

    let expected_name = manifest.worker_name(Some(env_name));

    let expected_routes = patterns
        .iter()
        .map(|p| Route {
            script: Some(expected_name.to_string()),
            pattern: p.to_string(),
            id: None,
        })
        .collect();

    let expected_deploy_target = DeployTarget::Zoned(Zoned {
        zone_id: zone_id.to_string(),
        routes: expected_routes,
    });

    assert_eq!(actual_deploy_target, expected_deploy_target);
}

#[test]
fn when_top_level_zoneless_env_empty() {
    let script = "top_level_zoneless_env_empty";
    let env_name = "test";
    let env_config = EnvConfig::default();

    let test_toml = WranglerToml::webpack_zoneless_with_env(script, true, env_name, env_config);
    let toml_string = toml::to_string(&test_toml).unwrap();
    let manifest = Manifest::new_from_string(toml_string).unwrap();

    let actual_deploy_target = manifest.deploy_target(Some(env_name)).unwrap();

    assert_eq!(actual_deploy_target, DeployTarget::Zoneless);
}

#[test]
fn when_top_level_zoneless_env_zoneless() {
    let env_name = "test";

    // TODO: SHOULD THIS TEST PASS?
    // when env.workers_dev = false
    let env_config = EnvConfig::zoneless(false);

    let script = "top_level_zoneless_env_zoneless_false";
    let test_toml = WranglerToml::webpack_zoneless_with_env(script, true, env_name, env_config);
    let toml_string = toml::to_string(&test_toml).unwrap();
    let manifest = Manifest::new_from_string(toml_string).unwrap();

    let actual_deploy_target = manifest.deploy_target(Some(env_name));

    assert!(actual_deploy_target.is_err());

    // when env.workers_dev = true
    let env_config = EnvConfig::zoneless(true);

    let script = "top_level_zoneless_env_zoneless_true";
    let test_toml = WranglerToml::webpack_zoneless_with_env(script, true, env_name, env_config);
    let toml_string = toml::to_string(&test_toml).unwrap();
    let manifest = Manifest::new_from_string(toml_string).unwrap();

    let actual_deploy_target = manifest.deploy_target(Some(env_name)).unwrap();

    assert_eq!(actual_deploy_target, DeployTarget::Zoneless);
}

#[test]
fn when_top_level_zoneless_env_zoned_single_route() {
    let env_name = "test";
    let zone_id = "samplezoneid";

    // when route is empty, error
    let pattern = "";
    let env_config = EnvConfig::zoned_single_route(zone_id, pattern);

    let script = "top_level_zoneless_env_zoned_single_route_empty";
    let test_toml = WranglerToml::webpack_zoneless_with_env(script, true, env_name, env_config);
    let toml_string = toml::to_string(&test_toml).unwrap();
    let manifest = Manifest::new_from_string(toml_string).unwrap();

    let actual_deploy_target = manifest.deploy_target(Some(env_name));

    assert!(actual_deploy_target.is_err());

    // when zone id is missing, error
    let pattern = "hostname.tld/*";
    let env_config = EnvConfig::zoned_single_route("", pattern);

    let script = "top_level_zoneless_env_zoned_single_route_no_zone_id";
    let test_toml = WranglerToml::webpack_zoneless_with_env(script, true, env_name, env_config);
    let toml_string = toml::to_string(&test_toml).unwrap();
    let manifest = Manifest::new_from_string(toml_string).unwrap();

    let actual_deploy_target = manifest.deploy_target(Some(env_name));

    assert!(actual_deploy_target.is_err());

    // when zone id is present, all good
    let pattern = "hostname.tld/*";
    let env_config = EnvConfig::zoned_single_route(zone_id, pattern);

    let script = "top_level_zoneless_env_zoned_single_route_no_zone_id";
    let test_toml = WranglerToml::webpack_zoneless_with_env(script, true, env_name, env_config);
    let toml_string = toml::to_string(&test_toml).unwrap();
    let manifest = Manifest::new_from_string(toml_string).unwrap();

    let actual_deploy_target = manifest.deploy_target(Some(env_name)).unwrap();

    let expected_name = manifest.worker_name(Some(env_name));

    let expected_routes = vec![Route {
        script: Some(expected_name),
        pattern: pattern.to_string(),
        id: None,
    }];
    let expected_deploy_target = DeployTarget::Zoned(Zoned {
        zone_id: zone_id.to_string(),
        routes: expected_routes,
    });

    assert_eq!(actual_deploy_target, expected_deploy_target);
}

#[test]
fn when_top_level_zoneless_env_zoned_multi_route() {
    let env_name = "test";
    let zone_id = "samplezoneid";

    // when routes list is empty, error
    let patterns = [];
    let env_config = EnvConfig::zoned_multi_route(zone_id, patterns.to_vec());

    let script = "top_level_zoneless_env_zoned_multi_route_empty_list";
    let test_toml = WranglerToml::webpack_zoneless_with_env(script, true, env_name, env_config);
    let toml_string = toml::to_string(&test_toml).unwrap();
    let manifest = Manifest::new_from_string(toml_string).unwrap();

    let actual_deploy_target = manifest.deploy_target(Some(env_name));

    assert!(actual_deploy_target.is_err());

    // when route is empty, error
    let patterns = [""];
    let env_config = EnvConfig::zoned_multi_route(zone_id, patterns.to_vec());

    let script = "top_level_zoneless_env_zoned_multi_route_empty";
    let test_toml = WranglerToml::webpack_zoneless_with_env(script, true, env_name, env_config);
    let toml_string = toml::to_string(&test_toml).unwrap();
    let manifest = Manifest::new_from_string(toml_string).unwrap();

    let actual_deploy_target = manifest.deploy_target(Some(env_name));

    assert!(actual_deploy_target.is_err());

    // when route key also present, error
    let patterns = ["hostname.tld/*"];
    let mut env_config = EnvConfig::zoned_multi_route(zone_id, patterns.to_vec());
    env_config.route = Some("blog.hostname.tld/*");

    let script = "top_level_zoneless_env_zoned_multi_route_empty";
    let test_toml = WranglerToml::webpack_zoneless_with_env(script, true, env_name, env_config);
    let toml_string = toml::to_string(&test_toml).unwrap();
    let manifest = Manifest::new_from_string(toml_string).unwrap();

    let actual_deploy_target = manifest.deploy_target(Some(env_name));

    assert!(actual_deploy_target.is_err());

    // when zone id is missing, error
    let patterns = ["hostname.tld/*"];
    let env_config = EnvConfig::zoned_multi_route("", patterns.to_vec());

    let script = "top_level_zoneless_env_zoned_multi_route_no_zone_id";
    let test_toml = WranglerToml::webpack_zoneless_with_env(script, true, env_name, env_config);
    let toml_string = toml::to_string(&test_toml).unwrap();
    let manifest = Manifest::new_from_string(toml_string).unwrap();

    let actual_deploy_target = manifest.deploy_target(Some(env_name));

    assert!(actual_deploy_target.is_err());

    // when zone id is present, all good
    let patterns = ["hostname.tld/*"];
    let env_config = EnvConfig::zoned_multi_route(zone_id, patterns.to_vec());

    let script = "top_level_zoneless_env_zoned_multi_route_no_zone_id";
    let test_toml = WranglerToml::webpack_zoneless_with_env(script, true, env_name, env_config);
    let toml_string = toml::to_string(&test_toml).unwrap();
    let manifest = Manifest::new_from_string(toml_string).unwrap();

    let actual_deploy_target = manifest.deploy_target(Some(env_name)).unwrap();

    let expected_name = manifest.worker_name(Some(env_name));

    let expected_routes = patterns
        .iter()
        .map(|p| Route {
            script: Some(expected_name.to_string()),
            pattern: p.to_string(),
            id: None,
        })
        .collect();

    let expected_deploy_target = DeployTarget::Zoned(Zoned {
        zone_id: zone_id.to_string(),
        routes: expected_routes,
    });

    assert_eq!(actual_deploy_target, expected_deploy_target);
}

#[test]
fn when_top_level_zoned_env_empty() {
    let zone_id = "samplezoneid";
    let pattern = "hostname.tld/*";
    let env_name = "test";

    let env_config = EnvConfig::default();

    let script = "top_level_zoned_env_empty";
    let test_toml = WranglerToml::webpack_zoned_single_route_with_env(
        script, zone_id, pattern, env_name, env_config,
    );
    let toml_string = toml::to_string(&test_toml).unwrap();
    let manifest = Manifest::new_from_string(toml_string).unwrap();

    let actual_deploy_target = manifest.deploy_target(Some(env_name)).unwrap();

    let expected_name = manifest.worker_name(Some(env_name));

    let expected_routes = vec![Route {
        script: Some(expected_name),
        pattern: pattern.to_string(),
        id: None,
    }];
    let expected_deploy_target = DeployTarget::Zoned(Zoned {
        zone_id: zone_id.to_string(),
        routes: expected_routes,
    });

    assert_eq!(actual_deploy_target, expected_deploy_target);
}

#[test]
fn when_top_level_zoned_env_zoneless() {
    let zone_id = "samplezoneid";
    let pattern = "hostname.tld/*";
    let env_name = "test";

    // TODO: SHOULD THIS TEST PASS?
    // when env.workers_dev = false
    let env_config = EnvConfig::zoneless(false);

    let script = "top_level_zoned_env_zoneless_false";
    let test_toml = WranglerToml::webpack_zoned_single_route_with_env(
        script, zone_id, pattern, env_name, env_config,
    );
    let toml_string = toml::to_string(&test_toml).unwrap();
    let manifest = Manifest::new_from_string(toml_string).unwrap();

    let actual_deploy_target = manifest.deploy_target(Some(env_name));

    assert!(actual_deploy_target.is_err());

    // when env.workers_dev = true
    let env_config = EnvConfig::zoneless(true);

    let script = "top_level_zoned_env_zoneless_true";
    let test_toml = WranglerToml::webpack_zoned_single_route_with_env(
        script, zone_id, pattern, env_name, env_config,
    );
    let toml_string = toml::to_string(&test_toml).unwrap();
    let manifest = Manifest::new_from_string(toml_string).unwrap();

    let actual_deploy_target = manifest.deploy_target(Some(env_name)).unwrap();

    assert_eq!(actual_deploy_target, DeployTarget::Zoneless);
}

#[test]
fn when_top_level_zoned_env_zoned_single_route() {
    let zone_id = "samplezoneid";
    let pattern = "hostname.tld/*";
    let env_name = "test";

    // when route is empty, error
    let env_pattern = "";
    let env_config = EnvConfig::zoned_single_route(zone_id, env_pattern);

    let script = "top_level_zoned_env_zoned_single_route_empty";
    let test_toml = WranglerToml::webpack_zoned_single_route_with_env(
        script, zone_id, pattern, env_name, env_config,
    );
    let toml_string = toml::to_string(&test_toml).unwrap();
    let manifest = Manifest::new_from_string(toml_string).unwrap();

    let actual_deploy_target = manifest.deploy_target(Some(env_name));

    assert!(actual_deploy_target.is_err());

    // when zone id is missing, use top level zone
    let env_pattern = "env.hostname.tld/*";
    let env_config = EnvConfig::zoned_single_route("", env_pattern);

    let script = "top_level_zoned_env_zoned_single_route_no_zone_id";
    let test_toml = WranglerToml::webpack_zoned_single_route_with_env(
        script, zone_id, pattern, env_name, env_config,
    );
    let toml_string = toml::to_string(&test_toml).unwrap();
    let manifest = Manifest::new_from_string(toml_string).unwrap();

    let actual_deploy_target = manifest.deploy_target(Some(env_name)).unwrap();

    let expected_name = manifest.worker_name(Some(env_name));

    let expected_routes = vec![Route {
        script: Some(expected_name),
        pattern: env_pattern.to_string(),
        id: None,
    }];
    let expected_deploy_target = DeployTarget::Zoned(Zoned {
        zone_id: zone_id.to_string(),
        routes: expected_routes,
    });

    assert_eq!(actual_deploy_target, expected_deploy_target);

    // when zone id is present in env, use that
    let env_pattern = "hostname.tld/*";
    let env_zone_id = "sampleenvzoneid";
    let env_config = EnvConfig::zoned_single_route(env_zone_id, env_pattern);

    let script = "top_level_zoned_env_zoned_single_route_no_zone_id";
    let test_toml = WranglerToml::webpack_zoned_single_route_with_env(
        script, zone_id, pattern, env_name, env_config,
    );
    let toml_string = toml::to_string(&test_toml).unwrap();
    let manifest = Manifest::new_from_string(toml_string).unwrap();

    let actual_deploy_target = manifest.deploy_target(Some(env_name)).unwrap();

    let expected_name = manifest.worker_name(Some(env_name));

    let expected_routes = vec![Route {
        script: Some(expected_name),
        pattern: pattern.to_string(),
        id: None,
    }];
    let expected_deploy_target = DeployTarget::Zoned(Zoned {
        zone_id: env_zone_id.to_string(),
        routes: expected_routes,
    });

    assert_eq!(actual_deploy_target, expected_deploy_target);
}
