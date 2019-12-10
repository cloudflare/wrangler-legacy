use super::wrangler_toml::WranglerToml;

use crate::settings::toml::deploy_target::Zoned;
use crate::settings::toml::route::Route;
use crate::settings::toml::DeployTarget;
use crate::settings::toml::Manifest;

// TOP LEVEL TESTS
#[test]
fn it_can_get_a_top_level_zoneless_deploy_target() {
    let test_toml = WranglerToml::webpack_no_config("zoneless");
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
fn it_errors_on_zoneless_deploy_target_workers_dev_false() {
    let mut test_toml = WranglerToml::webpack("zoneless_false");
    test_toml.workers_dev = Some(false);
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

    let mut test_toml = WranglerToml::webpack(script);
    test_toml.route = Some(pattern);
    test_toml.zone_id = Some(zone_id);
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

    let mut test_toml = WranglerToml::webpack(script);
    test_toml.workers_dev = Some(false);
    test_toml.route = Some(pattern);
    test_toml.zone_id = Some(zone_id);
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
fn it_errors_on_single_route_deploy_target_missing_zone_id() {
    let script = "single_route_missing_zone_id";
    let pattern = "hostname.tld/*";
    let zone_id = "";

    let mut test_toml = WranglerToml::webpack(script);
    test_toml.route = Some(pattern);
    test_toml.zone_id = Some(zone_id);
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

    let mut test_toml = WranglerToml::webpack(script);
    test_toml.route = Some(pattern);
    test_toml.zone_id = Some(zone_id);
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
fn it_errors_on_multi_route_deploy_target_missing_zone_id() {
    let script = "multi_route_missing_zone_id";
    let patterns = ["hostname.tld/*", "blog.hostname.tld/*"];
    let zone_id = "";

    let mut test_toml = WranglerToml::webpack(script);
    test_toml.routes = Some(patterns.to_vec());
    test_toml.zone_id = Some(zone_id);
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

    let mut test_toml = WranglerToml::webpack(script);
    test_toml.routes = Some(patterns.to_vec());
    test_toml.zone_id = Some(zone_id);
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

    let mut test_toml = WranglerToml::webpack(script);
    test_toml.routes = Some(patterns.to_vec());
    test_toml.zone_id = Some(zone_id);
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

    let mut test_toml = WranglerToml::webpack(script);
    test_toml.route = Some(pattern);
    test_toml.routes = Some(patterns.to_vec());
    test_toml.zone_id = Some(zone_id);
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

    let mut test_toml = WranglerToml::webpack(script);
    test_toml.workers_dev = Some(true);
    test_toml.route = Some(pattern);
    test_toml.zone_id = Some(zone_id);
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

    let mut test_toml = WranglerToml::webpack(script);
    test_toml.workers_dev = Some(true);
    test_toml.routes = Some(patterns.to_vec());
    test_toml.zone_id = Some(zone_id);
    let toml_string = toml::to_string(&test_toml).unwrap();
    let manifest = Manifest::new_from_string(toml_string).unwrap();

    let environment = None;

    assert!(manifest.deploy_target(environment).is_err());
}
