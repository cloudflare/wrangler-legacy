use super::wrangler_toml::{EnvConfig, WranglerToml, TEST_ENV_NAME};

use crate::settings::toml::route::Route;
use crate::settings::toml::Manifest;
use crate::settings::toml::{DeployConfig, Zoned, Zoneless};

// TOP LEVEL TESTS
#[test]
fn it_errors_on_empty_deploy_target() {
    let test_toml = WranglerToml::webpack("empty");
    let toml_string = toml::to_string(&test_toml).unwrap();
    let manifest = Manifest::new_from_string(toml_string).unwrap();

    let environment = None;

    assert!(manifest.deploy_config(environment).is_err());
}

#[test]
fn it_errors_on_conflicting_deploy_targets() {
    let script_name = "workers_dev_true_and_zoned_config";
    let pattern = "hostname.tld/*";
    let zone_id = "samplezoneid";
    let account_id = "fakeaccountid";
    let workers_dev = true;

    let mut test_toml = WranglerToml::zoned_single_route(script_name, zone_id, pattern);
    test_toml.workers_dev = Some(workers_dev);
    test_toml.account_id = Some(account_id);
    let toml_string = toml::to_string(&test_toml).unwrap();
    let manifest = Manifest::new_from_string(toml_string).unwrap();

    let environment = None;

    assert!(manifest.deploy_config(environment).is_err());
}

#[test]
fn it_can_get_a_top_level_zoneless_deploy_target() {
    let script_name = "zoneless";
    let account_id = "fakeaccountid";
    let workers_dev = true;
    let test_toml = WranglerToml::zoneless(script_name, account_id, workers_dev);
    let toml_string = toml::to_string(&test_toml).unwrap();
    let manifest = Manifest::new_from_string(toml_string).unwrap();

    let environment = None;
    let actual_deploy_target = manifest.deploy_config(environment).unwrap();
    let expected_deploy_target = DeployConfig::Zoneless(Zoneless {
        script_name: script_name.to_string(),
        account_id: account_id.to_string(),
    });

    assert_eq!(actual_deploy_target, expected_deploy_target);
}

#[test]
fn it_errors_on_deploy_target_missing_name() {
    let script_name = "";
    let account_id = "account_id";
    let workers_dev = true;
    let test_toml = WranglerToml::zoneless(script_name, account_id, workers_dev);
    let toml_string = toml::to_string(&test_toml).unwrap();
    let manifest = Manifest::new_from_string(toml_string).unwrap();

    let environment = None;

    assert!(manifest.deploy_config(environment).is_err());
}

#[test]
fn it_errors_on_deploy_target_missing_account_id() {
    let script_name = "zoneless_no_account_id";
    let account_id = "account_id";
    let workers_dev = true;
    let mut test_toml = WranglerToml::zoneless(script_name, account_id, workers_dev);
    test_toml.account_id = None;
    let toml_string = toml::to_string(&test_toml).unwrap();
    let manifest = Manifest::new_from_string(toml_string).unwrap();

    let environment = None;

    assert!(manifest.deploy_config(environment).is_err());
}

#[test]
fn it_errors_on_zoneless_deploy_target_workers_dev_false() {
    let script_name = "zoneless_false";
    let account_id = "fakeaccountid";
    let workers_dev = false;
    let test_toml = WranglerToml::zoneless(script_name, account_id, workers_dev);
    let toml_string = toml::to_string(&test_toml).unwrap();
    let manifest = Manifest::new_from_string(toml_string).unwrap();

    let environment = None;

    assert!(manifest.deploy_config(environment).is_err());
}

#[test]
fn it_can_get_a_single_route_zoned_deploy_target() {
    let script_name = "single_route_zoned";
    let pattern = "hostname.tld/*";
    let zone_id = "samplezoneid";

    let test_toml = WranglerToml::zoned_single_route(script_name, zone_id, pattern);
    let toml_string = toml::to_string(&test_toml).unwrap();
    let manifest = Manifest::new_from_string(toml_string).unwrap();

    let environment = None;
    let actual_deploy_target = manifest.deploy_config(environment).unwrap();
    let expected_routes = vec![Route {
        script: Some(script_name.to_string()),
        pattern: pattern.to_string(),
        id: None,
    }];
    let expected_deploy_target = DeployConfig::Zoned(Zoned {
        zone_id: zone_id.to_string(),
        routes: expected_routes,
    });

    assert_eq!(actual_deploy_target, expected_deploy_target);
}

#[test]
fn it_can_get_a_single_route_zoned_deploy_target_workers_dev_false() {
    let script_name = "single_route_zoned_workers_dev_false";
    let pattern = "hostname.tld/*";
    let zone_id = "samplezoneid";

    let mut test_toml = WranglerToml::zoned_single_route(script_name, zone_id, pattern);
    test_toml.workers_dev = Some(false);
    let toml_string = toml::to_string(&test_toml).unwrap();
    let manifest = Manifest::new_from_string(toml_string).unwrap();

    let environment = None;
    let actual_deploy_target = manifest.deploy_config(environment).unwrap();
    let expected_routes = vec![Route {
        script: Some(script_name.to_string()),
        pattern: pattern.to_string(),
        id: None,
    }];
    let expected_deploy_target = DeployConfig::Zoned(Zoned {
        zone_id: zone_id.to_string(),
        routes: expected_routes,
    });

    assert_eq!(actual_deploy_target, expected_deploy_target);
}

#[test]
fn it_errors_on_single_route_deploy_target_empty_zone_id() {
    let script_name = "single_route_empty_zone_id";
    let pattern = "hostname.tld/*";
    let zone_id = "";

    let test_toml = WranglerToml::zoned_single_route(script_name, zone_id, pattern);
    let toml_string = toml::to_string(&test_toml).unwrap();
    let manifest = Manifest::new_from_string(toml_string).unwrap();

    let environment = None;

    assert!(manifest.deploy_config(environment).is_err());
}

#[test]
fn it_errors_on_single_route_deploy_target_missing_zone_id() {
    let script_name = "single_route_empty_zone_id";
    let pattern = "hostname.tld/*";

    let mut test_toml = WranglerToml::zoned_single_route(script_name, "", pattern);
    test_toml.zone_id = None;
    let toml_string = toml::to_string(&test_toml).unwrap();
    let manifest = Manifest::new_from_string(toml_string).unwrap();

    let environment = None;

    assert!(manifest.deploy_config(environment).is_err());
}

#[test]
fn it_errors_on_single_route_deploy_target_empty_route() {
    let script_name = "single_route_empty_route";
    let pattern = "";
    let zone_id = "samplezoneid";

    let test_toml = WranglerToml::zoned_single_route(script_name, zone_id, pattern);
    let toml_string = toml::to_string(&test_toml).unwrap();
    let manifest = Manifest::new_from_string(toml_string).unwrap();

    let environment = None;

    assert!(manifest.deploy_config(environment).is_err());
}

#[test]
fn it_errors_on_single_route_deploy_target_missing_route() {
    let script_name = "single_route_missing_route";
    let zone_id = "samplezoneid";

    let mut test_toml = WranglerToml::zoned_single_route(script_name, zone_id, "");
    test_toml.route = None;
    let toml_string = toml::to_string(&test_toml).unwrap();
    let manifest = Manifest::new_from_string(toml_string).unwrap();

    let environment = None;

    assert!(manifest.deploy_config(environment).is_err());
}

#[test]
fn it_can_get_a_multi_route_zoned_deploy_target() {
    let script_name = "multi_route_zoned";
    let patterns = ["hostname.tld/*", "blog.hostname.tld/*"];
    let zone_id = "samplezoneid";

    let mut test_toml = WranglerToml::webpack(script_name);
    test_toml.routes = Some(patterns.to_vec());
    test_toml.zone_id = Some(zone_id);
    let toml_string = toml::to_string(&test_toml).unwrap();
    let manifest = Manifest::new_from_string(toml_string).unwrap();

    let expected_routes = patterns
        .iter()
        .map(|p| Route {
            script: Some(script_name.to_string()),
            pattern: p.to_string(),
            id: None,
        })
        .collect();
    let expected_deploy_target = DeployConfig::Zoned(Zoned {
        zone_id: zone_id.to_string(),
        routes: expected_routes,
    });

    let environment = None;
    let actual_deploy_target = manifest.deploy_config(environment).unwrap();

    assert_eq!(actual_deploy_target, expected_deploy_target);
}

#[test]
fn it_can_get_a_multi_route_zoned_deploy_target_workers_dev_false() {
    let script_name = "multi_route_zoned_workers_dev_false";
    let patterns = ["hostname.tld/*", "blog.hostname.tld/*"];
    let zone_id = "samplezoneid";

    let mut test_toml = WranglerToml::webpack(script_name);
    test_toml.workers_dev = Some(false);
    test_toml.routes = Some(patterns.to_vec());
    test_toml.zone_id = Some(zone_id);
    let toml_string = toml::to_string(&test_toml).unwrap();
    let manifest = Manifest::new_from_string(toml_string).unwrap();

    let expected_routes = patterns
        .iter()
        .map(|p| Route {
            script: Some(script_name.to_string()),
            pattern: p.to_string(),
            id: None,
        })
        .collect();
    let expected_deploy_target = DeployConfig::Zoned(Zoned {
        zone_id: zone_id.to_string(),
        routes: expected_routes,
    });

    let environment = None;
    let actual_deploy_target = manifest.deploy_config(environment).unwrap();

    assert_eq!(actual_deploy_target, expected_deploy_target);
}

#[test]
fn it_errors_on_multi_route_deploy_target_empty_zone_id() {
    let script_name = "multi_route_empty_zone_id";
    let patterns = ["hostname.tld/*", "blog.hostname.tld/*"];
    let zone_id = "";

    let test_toml = WranglerToml::zoned_multi_route(script_name, zone_id, patterns.to_vec());
    let toml_string = toml::to_string(&test_toml).unwrap();
    let manifest = Manifest::new_from_string(toml_string).unwrap();

    let environment = None;

    assert!(manifest.deploy_config(environment).is_err());
}

#[test]
fn it_errors_on_multi_route_deploy_target_missing_zone_id() {
    let script_name = "multi_route_missing_zone_id";
    let patterns = ["hostname.tld/*", "blog.hostname.tld/*"];
    let zone_id = "";

    let mut test_toml = WranglerToml::zoned_multi_route(script_name, zone_id, patterns.to_vec());
    test_toml.zone_id = None;
    let toml_string = toml::to_string(&test_toml).unwrap();
    let manifest = Manifest::new_from_string(toml_string).unwrap();

    let environment = None;

    assert!(manifest.deploy_config(environment).is_err());
}

#[test]
fn it_errors_on_multi_route_deploy_target_empty_routes_list() {
    let script_name = "multi_route_empty_routes_list";
    let patterns = [];
    let zone_id = "samplezoneid";

    let test_toml = WranglerToml::zoned_multi_route(script_name, zone_id, patterns.to_vec());
    let toml_string = toml::to_string(&test_toml).unwrap();
    let manifest = Manifest::new_from_string(toml_string).unwrap();

    let environment = None;

    assert!(manifest.deploy_config(environment).is_err());
}

#[test]
fn it_errors_on_multi_route_deploy_target_empty_route() {
    let script_name = "multi_route_empty_route";
    let patterns = [""];
    let zone_id = "samplezoneid";

    let test_toml = WranglerToml::zoned_multi_route(script_name, zone_id, patterns.to_vec());
    let toml_string = toml::to_string(&test_toml).unwrap();
    let manifest = Manifest::new_from_string(toml_string).unwrap();

    let environment = None;

    assert!(manifest.deploy_config(environment).is_err());
}

#[test]
fn it_errors_on_deploy_target_route_and_routes() {
    let script_name = "route_and_routes";
    let pattern = "hostname.tld/*";
    let patterns = ["blog.hostname.tld/*"];
    let zone_id = "samplezoneid";

    let mut test_toml = WranglerToml::zoned_single_route(script_name, zone_id, pattern);
    test_toml.routes = Some(patterns.to_vec());
    let toml_string = toml::to_string(&test_toml).unwrap();
    let manifest = Manifest::new_from_string(toml_string).unwrap();

    let environment = None;

    assert!(manifest.deploy_config(environment).is_err());
}

#[test]
fn it_errors_on_deploy_target_route_and_workers_dev_true() {
    let script_name = "route_and_workers_dev";
    let pattern = "hostname.tld/*";
    let zone_id = "samplezoneid";

    let mut test_toml = WranglerToml::zoned_single_route(script_name, zone_id, pattern);
    test_toml.workers_dev = Some(true);
    let toml_string = toml::to_string(&test_toml).unwrap();
    let manifest = Manifest::new_from_string(toml_string).unwrap();

    let environment = None;

    assert!(manifest.deploy_config(environment).is_err());
}

#[test]
fn it_errors_on_deploy_target_routes_and_workers_dev_true() {
    let script_name = "routes_and_workers_dev";
    let patterns = ["blog.hostname.tld/*"];
    let zone_id = "samplezoneid";

    let mut test_toml = WranglerToml::zoned_multi_route(script_name, zone_id, patterns.to_vec());
    test_toml.workers_dev = Some(true);
    let toml_string = toml::to_string(&test_toml).unwrap();
    let manifest = Manifest::new_from_string(toml_string).unwrap();

    let environment = None;

    assert!(manifest.deploy_config(environment).is_err());
}

// ENVIRONMENT TESTS
// Top level empty
#[test]
fn when_top_level_empty_env_empty() {
    let script_name = "top_level_empty_env_empty";
    let env_config = EnvConfig::default();

    let test_toml = WranglerToml::with_env(script_name, env_config);
    let toml_string = toml::to_string(&test_toml).unwrap();
    let manifest = Manifest::new_from_string(toml_string).unwrap();

    let actual_deploy_target = manifest.deploy_config(Some(TEST_ENV_NAME));

    assert!(actual_deploy_target.is_err());
}

#[test]
fn when_top_level_empty_env_has_zone_id() {
    // if env only includes zone id, error
    let script_name = "when_top_level_empty_env_has_zone_id";
    let mut env_config = EnvConfig::default();
    env_config.zone_id = Some("samplezoneid");

    let test_toml = WranglerToml::with_env(script_name, env_config);
    let toml_string = toml::to_string(&test_toml).unwrap();
    let manifest = Manifest::new_from_string(toml_string).unwrap();

    let actual_deploy_target = manifest.deploy_config(Some(TEST_ENV_NAME));

    assert!(actual_deploy_target.is_err());
}

#[test]
fn when_top_level_empty_env_workers_dev_false() {
    let account_id = "testaccountid";
    let workers_dev = false;

    let env_config = EnvConfig::zoneless_with_account_id(workers_dev, account_id);

    let script_name = "top_level_empty_env_empty";
    let test_toml = WranglerToml::with_env(script_name, env_config);
    let toml_string = toml::to_string(&test_toml).unwrap();
    let manifest = Manifest::new_from_string(toml_string).unwrap();

    let actual_deploy_target = manifest.deploy_config(Some(TEST_ENV_NAME));

    assert!(actual_deploy_target.is_err());
}

#[test]
fn when_top_level_empty_env_workers_dev_true() {
    let account_id = "testaccountid";
    let workers_dev = true;
    let env_config = EnvConfig::zoneless_with_account_id(workers_dev, account_id);

    let script_name = "top_level_empty_env_zoneless_true";
    let test_toml = WranglerToml::with_env(script_name, env_config);
    let toml_string = toml::to_string(&test_toml).unwrap();
    let manifest = Manifest::new_from_string(toml_string).unwrap();

    let environment = Some(TEST_ENV_NAME);
    let actual_deploy_target = manifest.deploy_config(environment).unwrap();
    let expected_deploy_target = DeployConfig::Zoneless(Zoneless {
        script_name: manifest.worker_name(environment),
        account_id: account_id.to_string(),
    });

    assert_eq!(actual_deploy_target, expected_deploy_target);
}

#[test]
fn when_top_level_empty_zoned_single_route_env() {
    let zone_id = "samplezoneid";

    // when route is empty, error
    let pattern = "";
    let env_config = EnvConfig::zoned_single_route(zone_id, pattern);

    let script_name = "top_level_empty_env_zoned_single_route_empty";
    let test_toml = WranglerToml::with_env(script_name, env_config);
    let toml_string = toml::to_string(&test_toml).unwrap();
    let manifest = Manifest::new_from_string(toml_string).unwrap();

    let actual_deploy_target = manifest.deploy_config(Some(TEST_ENV_NAME));

    assert!(actual_deploy_target.is_err());
}

#[test]
fn when_top_level_empty_env_zoned_single_route_no_zone_id() {
    let pattern = "hostname.tld/*";
    let env_config = EnvConfig::zoned_single_route("", pattern);

    let script_name = "top_level_empty_env_zoned_single_route_no_zone_id";
    let test_toml = WranglerToml::with_env(script_name, env_config);
    let toml_string = toml::to_string(&test_toml).unwrap();
    let manifest = Manifest::new_from_string(toml_string).unwrap();

    let actual_deploy_target = manifest.deploy_config(Some(TEST_ENV_NAME));

    assert!(actual_deploy_target.is_err());
}

#[test]
fn when_top_level_empty_env_zoned_single_route_zone_id_only() {
    let zone_id = "samplezoneid";
    let mut env_config = EnvConfig::default();
    env_config.zone_id = Some(zone_id);

    let script_name = "top_level_empty_env_zoned_single_route_zone_id_only";
    let test_toml = WranglerToml::with_env(script_name, env_config);
    let toml_string = toml::to_string(&test_toml).unwrap();
    let manifest = Manifest::new_from_string(toml_string).unwrap();

    let actual_deploy_target = manifest.deploy_config(Some(TEST_ENV_NAME));

    assert!(actual_deploy_target.is_err());
}

#[test]
fn when_top_level_empty_env_zoned_single_route() {
    let zone_id = "samplezoneid";
    let pattern = "hostname.tld/*";
    let env_config = EnvConfig::zoned_single_route(zone_id, pattern);

    let script_name = "top_level_empty_env_zoned_single_route_no_zone_id";
    let test_toml = WranglerToml::with_env(script_name, env_config);
    let toml_string = toml::to_string(&test_toml).unwrap();
    let manifest = Manifest::new_from_string(toml_string).unwrap();

    let actual_deploy_target = manifest.deploy_config(Some(TEST_ENV_NAME)).unwrap();

    let expected_name = manifest.worker_name(Some(TEST_ENV_NAME));

    let expected_routes = vec![Route {
        script: Some(expected_name),
        pattern: pattern.to_string(),
        id: None,
    }];
    let expected_deploy_target = DeployConfig::Zoned(Zoned {
        zone_id: zone_id.to_string(),
        routes: expected_routes,
    });

    assert_eq!(actual_deploy_target, expected_deploy_target);
}

#[test]
fn when_top_level_empty_zoned_multi_route_env_routes_empty() {
    let zone_id = "samplezoneid";

    // when routes list is empty, error
    let patterns = [];
    let env_config = EnvConfig::zoned_multi_route(zone_id, patterns.to_vec());

    let script_name = "top_level_empty_zoned_multi_route_env_routes_empty";
    let test_toml = WranglerToml::with_env(script_name, env_config);
    let toml_string = toml::to_string(&test_toml).unwrap();
    let manifest = Manifest::new_from_string(toml_string).unwrap();

    let actual_deploy_target = manifest.deploy_config(Some(TEST_ENV_NAME));

    assert!(actual_deploy_target.is_err());
}

#[test]
fn when_top_level_empty_zoned_multi_route_env_route_empty() {
    let zone_id = "samplezoneid";

    // when route is empty, error
    let patterns = [""];
    let env_config = EnvConfig::zoned_multi_route(zone_id, patterns.to_vec());

    let script_name = "top_level_empty_env_zoned_multi_route_empty";
    let test_toml = WranglerToml::with_env(script_name, env_config);
    let toml_string = toml::to_string(&test_toml).unwrap();
    let manifest = Manifest::new_from_string(toml_string).unwrap();

    let actual_deploy_target = manifest.deploy_config(Some(TEST_ENV_NAME));

    assert!(actual_deploy_target.is_err());
}

#[test]
fn when_top_level_empty_zoned_multi_route_env_zone_id_missing() {
    // when zone id is missing, error
    let patterns = ["hostname.tld/*"];
    let env_config = EnvConfig::zoned_multi_route("", patterns.to_vec());

    let script_name = "top_level_empty_zoned_multi_route_env_zone_id_missing";
    let test_toml = WranglerToml::with_env(script_name, env_config);
    let toml_string = toml::to_string(&test_toml).unwrap();
    let manifest = Manifest::new_from_string(toml_string).unwrap();

    let actual_deploy_target = manifest.deploy_config(Some(TEST_ENV_NAME));

    assert!(actual_deploy_target.is_err());
}

#[test]
fn when_top_level_empty_zoned_multi_route_env() {
    let zone_id = "samplezoneid";

    // when zone id is present, all good
    let patterns = ["hostname.tld/*"];
    let env_config = EnvConfig::zoned_multi_route(zone_id, patterns.to_vec());

    let script_name = "top_level_empty_env_zoned_multi_route_no_zone_id";
    let test_toml = WranglerToml::with_env(script_name, env_config);
    let toml_string = toml::to_string(&test_toml).unwrap();
    let manifest = Manifest::new_from_string(toml_string).unwrap();

    let actual_deploy_target = manifest.deploy_config(Some(TEST_ENV_NAME)).unwrap();

    let expected_name = manifest.worker_name(Some(TEST_ENV_NAME));

    let expected_routes = patterns
        .iter()
        .map(|p| Route {
            script: Some(expected_name.to_string()),
            pattern: p.to_string(),
            id: None,
        })
        .collect();

    let expected_deploy_target = DeployConfig::Zoned(Zoned {
        zone_id: zone_id.to_string(),
        routes: expected_routes,
    });

    assert_eq!(actual_deploy_target, expected_deploy_target);
}

#[test]
fn when_top_level_zoneless_env_empty() {
    let script_name = "top_level_zoneless_env_empty";
    let account_id = "account_id";
    let env_config = EnvConfig::default();
    let workers_dev = true;

    let test_toml =
        WranglerToml::zoneless_with_env(script_name, account_id, workers_dev, env_config);
    let toml_string = toml::to_string(&test_toml).unwrap();
    let manifest = Manifest::new_from_string(toml_string).unwrap();

    let environment = Some(TEST_ENV_NAME);
    let actual_deploy_target = manifest.deploy_config(environment).unwrap();
    let expected_deploy_target = DeployConfig::Zoneless(Zoneless {
        script_name: manifest.worker_name(environment),
        account_id: account_id.to_string(),
    });

    assert_eq!(actual_deploy_target, expected_deploy_target);
}

#[test]
fn when_top_level_zoneless_env_zoneless_workers_dev_false() {
    let account_id = "account_id";
    let env_config = EnvConfig::zoneless(false);

    let script_name = "top_level_zoneless_env_zoneless_workers_dev_false";
    let workers_dev = true;
    let test_toml =
        WranglerToml::zoneless_with_env(script_name, account_id, workers_dev, env_config);
    let toml_string = toml::to_string(&test_toml).unwrap();
    let manifest = Manifest::new_from_string(toml_string).unwrap();

    let actual_deploy_target = manifest.deploy_config(Some(TEST_ENV_NAME));

    assert!(actual_deploy_target.is_err());
}

#[test]
fn when_top_level_zoneless_env_zoneless_workers_dev_true() {
    let account_id = "account_id";
    // when env.workers_dev = true
    let env_config = EnvConfig::zoneless(true);

    let script_name = "top_level_zoneless_env_zoneless_workers_dev_false";
    let workers_dev = true;
    let test_toml =
        WranglerToml::zoneless_with_env(script_name, account_id, workers_dev, env_config);
    let toml_string = toml::to_string(&test_toml).unwrap();
    let manifest = Manifest::new_from_string(toml_string).unwrap();

    let environment = Some(TEST_ENV_NAME);
    let actual_deploy_target = manifest.deploy_config(environment).unwrap();
    let expected_deploy_target = DeployConfig::Zoneless(Zoneless {
        account_id: account_id.to_string(),
        script_name: manifest.worker_name(environment),
    });

    assert_eq!(actual_deploy_target, expected_deploy_target);
}

#[test]
fn when_top_level_zoneless_env_zoned_single_route_empty() {
    let account_id = "account_id";
    let zone_id = "samplezoneid";

    // when route is empty, error
    let pattern = "";
    let env_config = EnvConfig::zoned_single_route(zone_id, pattern);

    let script_name = "top_level_zoneless_env_zoned_single_route_empty";
    let workers_dev = true;
    let test_toml =
        WranglerToml::zoneless_with_env(script_name, account_id, workers_dev, env_config);
    let toml_string = toml::to_string(&test_toml).unwrap();
    let manifest = Manifest::new_from_string(toml_string).unwrap();

    let environment = Some(TEST_ENV_NAME);
    let actual_deploy_target = manifest.deploy_config(environment).unwrap();
    let expected_deploy_target = DeployConfig::Zoneless(Zoneless {
        account_id: account_id.to_string(),
        script_name: manifest.worker_name(environment),
    });

    assert_eq!(actual_deploy_target, expected_deploy_target);
}

#[test]
fn when_top_level_zoneless_env_zoned_single_route_zone_id_missing() {
    let account_id = "account_id";

    // when zone id is missing, error
    let pattern = "hostname.tld/*";
    let env_config = EnvConfig::zoned_single_route("", pattern);

    let script_name = "top_level_zoneless_env_zoned_single_route_zone_id_missing";
    let workers_dev = true;
    let test_toml =
        WranglerToml::zoneless_with_env(script_name, account_id, workers_dev, env_config);
    let toml_string = toml::to_string(&test_toml).unwrap();
    let manifest = Manifest::new_from_string(toml_string).unwrap();

    let actual_deploy_target = manifest.deploy_config(Some(TEST_ENV_NAME));

    assert!(actual_deploy_target.is_err());
}

#[test]
fn when_top_level_zoneless_env_zoned_single_route() {
    let account_id = "account_id";
    let zone_id = "samplezoneid";

    // when zone id is present, all good
    let pattern = "hostname.tld/*";
    let env_config = EnvConfig::zoned_single_route(zone_id, pattern);

    let script_name = "top_level_zoneless_env_zoned_single_route_no_zone_id";
    let workers_dev = true;
    let test_toml =
        WranglerToml::zoneless_with_env(script_name, account_id, workers_dev, env_config);
    let toml_string = toml::to_string(&test_toml).unwrap();
    let manifest = Manifest::new_from_string(toml_string).unwrap();

    let actual_deploy_target = manifest.deploy_config(Some(TEST_ENV_NAME)).unwrap();

    let expected_name = manifest.worker_name(Some(TEST_ENV_NAME));

    let expected_routes = vec![Route {
        script: Some(expected_name),
        pattern: pattern.to_string(),
        id: None,
    }];
    let expected_deploy_target = DeployConfig::Zoned(Zoned {
        zone_id: zone_id.to_string(),
        routes: expected_routes,
    });

    assert_eq!(actual_deploy_target, expected_deploy_target);
}

#[test]
fn when_top_level_zoneless_env_zoned_multi_route_routes_list_empty() {
    let zone_id = "samplezoneid";
    let account_id = "account_id";

    // when routes list is empty, error
    let patterns = [];
    let env_config = EnvConfig::zoned_multi_route(zone_id, patterns.to_vec());

    let script_name = "top_level_zoneless_env_zoned_multi_route_routes_list_empty";
    let workers_dev = true;
    let test_toml =
        WranglerToml::zoneless_with_env(script_name, account_id, workers_dev, env_config);
    let toml_string = toml::to_string(&test_toml).unwrap();
    let manifest = Manifest::new_from_string(toml_string).unwrap();

    let actual_deploy_target = manifest.deploy_config(Some(TEST_ENV_NAME));

    assert!(actual_deploy_target.is_err());
}

#[test]
fn when_top_level_zoneless_env_zoned_multi_route_route_empty() {
    let zone_id = "samplezoneid";
    let account_id = "account_id";

    // when route is empty, error
    let patterns = [""];
    let env_config = EnvConfig::zoned_multi_route(zone_id, patterns.to_vec());

    let script_name = "top_level_zoneless_env_zoned_multi_route_route_empty";
    let workers_dev = true;
    let test_toml =
        WranglerToml::zoneless_with_env(script_name, account_id, workers_dev, env_config);
    let toml_string = toml::to_string(&test_toml).unwrap();
    let manifest = Manifest::new_from_string(toml_string).unwrap();

    let actual_deploy_target = manifest.deploy_config(Some(TEST_ENV_NAME));

    assert!(actual_deploy_target.is_err());
}

#[test]
fn when_top_level_zoneless_env_zoned_multi_route_route_key_present() {
    let zone_id = "samplezoneid";
    let account_id = "account_id";

    // when route key also present, error
    let patterns = ["hostname.tld/*"];
    let mut env_config = EnvConfig::zoned_multi_route(zone_id, patterns.to_vec());
    env_config.route = Some("blog.hostname.tld/*");

    let script_name = "top_level_zoneless_env_zoned_multi_route_route_key_present";
    let workers_dev = true;
    let test_toml =
        WranglerToml::zoneless_with_env(script_name, account_id, workers_dev, env_config);
    let toml_string = toml::to_string(&test_toml).unwrap();
    let manifest = Manifest::new_from_string(toml_string).unwrap();

    let actual_deploy_target = manifest.deploy_config(Some(TEST_ENV_NAME));

    assert!(actual_deploy_target.is_err());
}

#[test]
fn when_top_level_zoneless_env_zoned_multi_route_zone_id_missing() {
    let account_id = "account_id";

    // when zone id is missing, error
    let patterns = ["hostname.tld/*"];
    let env_config = EnvConfig::zoned_multi_route("", patterns.to_vec());

    let script_name = "when_top_level_zoneless_env_zoned_multi_route_zone_id_missing";
    let workers_dev = true;
    let test_toml =
        WranglerToml::zoneless_with_env(script_name, account_id, workers_dev, env_config);
    let toml_string = toml::to_string(&test_toml).unwrap();
    let manifest = Manifest::new_from_string(toml_string).unwrap();

    let actual_deploy_target = manifest.deploy_config(Some(TEST_ENV_NAME));

    assert!(actual_deploy_target.is_err());
}

#[test]
fn when_top_level_zoneless_env_zoned_multi_route() {
    let zone_id = "samplezoneid";
    let account_id = "account_id";

    // when zone id is present, all good
    let patterns = ["hostname.tld/*"];
    let env_config = EnvConfig::zoned_multi_route(zone_id, patterns.to_vec());

    let script_name = "top_level_zoneless_env_zoned_multi_route_no_zone_id";
    let workers_dev = true;
    let test_toml =
        WranglerToml::zoneless_with_env(script_name, account_id, workers_dev, env_config);
    let toml_string = toml::to_string(&test_toml).unwrap();
    let manifest = Manifest::new_from_string(toml_string).unwrap();

    let actual_deploy_target = manifest.deploy_config(Some(TEST_ENV_NAME)).unwrap();

    let expected_name = manifest.worker_name(Some(TEST_ENV_NAME));

    let expected_routes = patterns
        .iter()
        .map(|p| Route {
            script: Some(expected_name.to_string()),
            pattern: p.to_string(),
            id: None,
        })
        .collect();

    let expected_deploy_target = DeployConfig::Zoned(Zoned {
        zone_id: zone_id.to_string(),
        routes: expected_routes,
    });

    assert_eq!(actual_deploy_target, expected_deploy_target);
}

#[test]
fn when_top_level_zoned_env_empty() {
    let zone_id = "samplezoneid";
    let pattern = "hostname.tld/*";

    let env_config = EnvConfig::default();

    let script_name = "top_level_zoned_env_empty";
    let test_toml =
        WranglerToml::zoned_single_route_with_env(script_name, zone_id, pattern, env_config);
    let toml_string = toml::to_string(&test_toml).unwrap();
    let manifest = Manifest::new_from_string(toml_string).unwrap();

    let actual_deploy_target = manifest.deploy_config(Some(TEST_ENV_NAME));

    assert!(actual_deploy_target.is_err());
}

#[test]
fn when_top_level_zoned_env_zoneless_workers_dev_false() {
    let zone_id = "samplezoneid";
    let pattern = "hostname.tld/*";
    let account_id = "account_id";

    // when env.workers_dev = false
    let workers_dev = false;
    let env_config = EnvConfig::zoneless_with_account_id(workers_dev, account_id);

    let script_name = "top_level_zoned_env_zoneless_workers_dev_false";
    let test_toml =
        WranglerToml::zoned_single_route_with_env(script_name, zone_id, pattern, env_config);
    let toml_string = toml::to_string(&test_toml).unwrap();
    let manifest = Manifest::new_from_string(toml_string).unwrap();

    let actual_deploy_target = manifest.deploy_config(Some(TEST_ENV_NAME));

    assert!(actual_deploy_target.is_err());
}

#[test]
fn when_top_level_zoned_env_zoneless_workers_dev_true() {
    let zone_id = "samplezoneid";
    let pattern = "hostname.tld/*";
    let account_id = "account_id";

    // when env.workers_dev = true
    let workers_dev = true;
    let env_config = EnvConfig::zoneless_with_account_id(workers_dev, account_id);

    let script_name = "when_top_level_zoned_env_zoneless_workers_dev_true";
    let test_toml =
        WranglerToml::zoned_single_route_with_env(script_name, zone_id, pattern, env_config);
    let toml_string = toml::to_string(&test_toml).unwrap();
    let manifest = Manifest::new_from_string(toml_string).unwrap();

    let environment = Some(TEST_ENV_NAME);
    let actual_deploy_target = manifest.deploy_config(environment).unwrap();
    let expected_deploy_target = DeployConfig::Zoneless(Zoneless {
        account_id: account_id.to_string(),
        script_name: manifest.worker_name(environment),
    });

    assert_eq!(actual_deploy_target, expected_deploy_target);
}

#[test]
fn when_top_level_zoned_env_zoned_single_route_route_empty() {
    let zone_id = "samplezoneid";
    let pattern = "hostname.tld/*";

    // when route is empty, error
    let env_pattern = "";
    let env_config = EnvConfig::zoned_single_route(zone_id, env_pattern);

    let script_name = "top_level_zoned_env_zoned_single_route_empty";
    let test_toml =
        WranglerToml::zoned_single_route_with_env(script_name, zone_id, pattern, env_config);
    let toml_string = toml::to_string(&test_toml).unwrap();
    let manifest = Manifest::new_from_string(toml_string).unwrap();

    let actual_deploy_target = manifest.deploy_config(Some(TEST_ENV_NAME));

    assert!(actual_deploy_target.is_err());
}

#[test]
fn when_top_level_zoned_env_zoned_single_route_zone_id_missing() {
    let zone_id = "samplezoneid";
    let pattern = "hostname.tld/*";

    // when zone id is missing, use top level zone
    let env_pattern = "env.hostname.tld/*";
    let env_config = EnvConfig::zoned_single_route("", env_pattern);

    let script_name = "top_level_zoned_env_zoned_single_route_no_zone_id";
    let test_toml =
        WranglerToml::zoned_single_route_with_env(script_name, zone_id, pattern, env_config);
    let toml_string = toml::to_string(&test_toml).unwrap();
    let manifest = Manifest::new_from_string(toml_string).unwrap();

    let actual_deploy_target = manifest.deploy_config(Some(TEST_ENV_NAME)).unwrap();

    let expected_name = manifest.worker_name(Some(TEST_ENV_NAME));

    let expected_routes = vec![Route {
        script: Some(expected_name),
        pattern: env_pattern.to_string(),
        id: None,
    }];
    let expected_deploy_target = DeployConfig::Zoned(Zoned {
        zone_id: zone_id.to_string(),
        routes: expected_routes,
    });

    assert_eq!(actual_deploy_target, expected_deploy_target);
}

#[test]
fn when_top_level_zoned_env_zoned_single_route() {
    let zone_id = "samplezoneid";
    let pattern = "hostname.tld/*";

    // when zone id is present in env, use that
    let env_pattern = "hostname.tld/*";
    let env_zone_id = "sampleenvzoneid";
    let env_config = EnvConfig::zoned_single_route(env_zone_id, env_pattern);

    let script_name = "top_level_zoned_env_zoned_single_route_no_zone_id";
    let test_toml =
        WranglerToml::zoned_single_route_with_env(script_name, zone_id, pattern, env_config);
    let toml_string = toml::to_string(&test_toml).unwrap();
    let manifest = Manifest::new_from_string(toml_string).unwrap();

    let actual_deploy_target = manifest.deploy_config(Some(TEST_ENV_NAME)).unwrap();

    let expected_name = manifest.worker_name(Some(TEST_ENV_NAME));

    let expected_routes = vec![Route {
        script: Some(expected_name),
        pattern: pattern.to_string(),
        id: None,
    }];
    let expected_deploy_target = DeployConfig::Zoned(Zoned {
        zone_id: env_zone_id.to_string(),
        routes: expected_routes,
    });

    assert_eq!(actual_deploy_target, expected_deploy_target);
}
