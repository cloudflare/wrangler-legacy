use std::str::FromStr;

use crate::deploy::{DeployTarget, ScheduleTarget, ZonedTarget, ZonelessTarget};
use crate::settings::toml::route::Route;
use crate::settings::toml::Manifest;

use super::fixtures::{EnvConfig, Triggers, WranglerToml, TEST_ENV_NAME};

// Test consts
const ZONE_ID: &str = "samplezoneid";
const PATTERN: &str = "hostname.tld/*";
const ACCOUNT_ID: &str = "fakeaccountid";

// TOP LEVEL TESTS
#[test]
fn it_errors_on_empty_get_deployments() {
    let test_toml = WranglerToml::webpack("empty");
    let toml_string = toml::to_string(&test_toml).unwrap();
    let manifest = Manifest::from_str(&toml_string).unwrap();

    let environment = None;

    assert!(manifest.get_deployments(environment).is_err());
}

#[test]
fn it_can_get_multiple_deployments() {
    let script_name = "workers_dev_true_and_zoned_config";
    let workers_dev = true;

    let mut test_toml = WranglerToml::zoned_single_route(script_name, ZONE_ID, PATTERN);
    test_toml.workers_dev = Some(workers_dev);
    test_toml.account_id = Some(ACCOUNT_ID);
    let toml_string = toml::to_string(&test_toml).unwrap();
    let manifest = Manifest::from_str(&toml_string).unwrap();

    let expected_deployments = vec![
        DeployTarget::Zoned(ZonedTarget {
            routes: vec![Route {
                script: Some(script_name.to_owned()),
                pattern: PATTERN.to_owned(),
                id: None,
            }],
            zone_id: ZONE_ID.to_owned(),
        }),
        DeployTarget::Zoneless(ZonelessTarget {
            account_id: ACCOUNT_ID.to_owned(),
            script_name: script_name.to_owned(),
        }),
    ];
    let environment = None;
    let actual_deployments = manifest.get_deployments(environment).unwrap();

    assert_eq!(actual_deployments, expected_deployments);
}

#[test]
fn it_can_get_a_top_level_zoneless_get_deployments() {
    let script_name = "zoneless";
    let workers_dev = true;
    let test_toml = WranglerToml::zoneless(script_name, ACCOUNT_ID, workers_dev);
    let toml_string = toml::to_string(&test_toml).unwrap();
    let manifest = Manifest::from_str(&toml_string).unwrap();

    let environment = None;
    let actual_deployments = manifest.get_deployments(environment).unwrap();
    let expected_deployments = vec![DeployTarget::Zoneless(ZonelessTarget {
        script_name: script_name.to_string(),
        account_id: ACCOUNT_ID.to_string(),
    })];

    assert_eq!(actual_deployments, expected_deployments);
}

#[test]
fn it_errors_on_get_deployments_missing_name() {
    let script_name = "";
    let workers_dev = true;
    let test_toml = WranglerToml::zoneless(script_name, ACCOUNT_ID, workers_dev);
    let toml_string = toml::to_string(&test_toml).unwrap();
    let manifest = Manifest::from_str(&toml_string).unwrap();

    let environment = None;

    assert!(manifest.get_deployments(environment).is_err());
}

#[test]
fn it_errors_on_get_deployments_missing_account_id() {
    let script_name = "zoneless_no_account_id";
    let workers_dev = true;
    let mut test_toml = WranglerToml::zoneless(script_name, ACCOUNT_ID, workers_dev);
    test_toml.account_id = None;
    let toml_string = toml::to_string(&test_toml).unwrap();
    let manifest = Manifest::from_str(&toml_string).unwrap();

    let environment = None;

    assert!(manifest.get_deployments(environment).is_err());
}

#[test]
fn it_errors_on_zoneless_get_deployments_workers_dev_false() {
    let script_name = "zoneless_false";
    let workers_dev = false;
    let test_toml = WranglerToml::zoneless(script_name, ACCOUNT_ID, workers_dev);
    let toml_string = toml::to_string(&test_toml).unwrap();
    let manifest = Manifest::from_str(&toml_string).unwrap();

    let environment = None;

    assert!(manifest.get_deployments(environment).is_err());
}

#[test]
fn it_can_get_a_single_route_zoned_get_deployments() {
    let script_name = "single_route_zoned";

    let test_toml = WranglerToml::zoned_single_route(script_name, ZONE_ID, PATTERN);
    let toml_string = toml::to_string(&test_toml).unwrap();
    let manifest = Manifest::from_str(&toml_string).unwrap();

    let environment = None;
    let actual_deployments = manifest.get_deployments(environment).unwrap();
    let expected_routes = vec![Route {
        script: Some(script_name.to_string()),
        pattern: PATTERN.to_string(),
        id: None,
    }];
    let expected_deployments = vec![DeployTarget::Zoned(ZonedTarget {
        zone_id: ZONE_ID.to_string(),
        routes: expected_routes,
    })];

    assert_eq!(actual_deployments, expected_deployments);
}

#[test]
fn it_can_get_a_single_route_zoned_get_deployments_workers_dev_false() {
    let script_name = "single_route_zoned_workers_dev_false";

    let mut test_toml = WranglerToml::zoned_single_route(script_name, ZONE_ID, PATTERN);
    test_toml.workers_dev = Some(false);
    let toml_string = toml::to_string(&test_toml).unwrap();
    let manifest = Manifest::from_str(&toml_string).unwrap();

    let expected_routes = vec![Route {
        script: Some(script_name.to_string()),
        pattern: PATTERN.to_string(),
        id: None,
    }];
    let expected_deployments = vec![DeployTarget::Zoned(ZonedTarget {
        zone_id: ZONE_ID.to_string(),
        routes: expected_routes,
    })];
    let environment = None;
    let actual_deployments = manifest.get_deployments(environment).unwrap();

    assert_eq!(actual_deployments, expected_deployments);
}

#[test]
fn it_can_get_a_scheduled_no_workers_dev_no_zoned() {
    let script_name = "single_schedule";

    let crons = vec!["0 * * * *".to_owned()];

    let mut test_toml = WranglerToml::webpack(script_name);
    test_toml.account_id = Some(ACCOUNT_ID);
    test_toml.triggers = Some(Triggers {
        crons: Some(crons.clone()),
    });

    let toml_string = toml::to_string(&test_toml).unwrap();
    let manifest = Manifest::from_str(&toml_string).unwrap();

    let expected_deployments = vec![DeployTarget::Schedule(ScheduleTarget {
        account_id: ACCOUNT_ID.to_owned(),
        script_name: script_name.to_owned(),
        crons,
    })];
    let environment = None;
    let actual_deployments = manifest.get_deployments(environment).unwrap();

    assert_eq!(actual_deployments, expected_deployments);
}

#[test]
fn it_can_get_a_scheduled_in_env_no_workers_dev_no_zoned() {
    let script_name = "single_schedule";

    let env_crons = vec!["0 * * * *".to_owned()];

    let env_config = EnvConfig {
        triggers: Some(Triggers {
            crons: Some(env_crons.clone()),
        }),
        ..EnvConfig::default()
    };
    let mut test_toml = WranglerToml {
        account_id: Some(ACCOUNT_ID),
        triggers: Some(Triggers {
            crons: Some(vec!["0 * * * *".to_owned()]),
        }),
        ..WranglerToml::webpack(script_name)
    };
    test_toml
        .env
        .get_or_insert_with(Default::default)
        .insert("b", env_config);

    let toml_string = toml::to_string(&test_toml).unwrap();
    let manifest = Manifest::from_str(&toml_string).unwrap();

    let environment = Some("b");
    let worker_name = manifest.worker_name(Some("b"));
    let expected_deployments = vec![DeployTarget::Schedule(ScheduleTarget {
        account_id: ACCOUNT_ID.to_owned(),
        script_name: worker_name,
        crons: env_crons,
    })];
    let actual_deployments = manifest.get_deployments(environment).unwrap();

    assert_eq!(actual_deployments, expected_deployments);
}
#[test]
fn it_cat_get_inherited_env_schedules() {
    // with no zoned, zoneless, or schedule targets in environment, we error
    let script_name = "single_schedule";

    let crons = vec!["0 * * * *".to_owned()];
    let env = EnvConfig::custom_script_name("inherited_schedule");

    let mut test_toml = WranglerToml::webpack(script_name);
    test_toml.account_id = Some(ACCOUNT_ID);
    test_toml.triggers = Some(Triggers {
        crons: Some(crons.clone()),
    });
    test_toml
        .env
        .get_or_insert_with(Default::default)
        .insert("b", env);

    let toml_string = toml::to_string(&test_toml).unwrap();
    let manifest = Manifest::from_str(&toml_string).unwrap();

    let environment = Some("b");
    let expected_deployments = vec![DeployTarget::Schedule(ScheduleTarget {
        account_id: ACCOUNT_ID.to_owned(),
        script_name: "inherited_schedule".to_owned(),
        crons,
    })];
    let actual_deployments = manifest.get_deployments(environment).unwrap();

    assert_eq!(actual_deployments, expected_deployments);
}

#[test]
fn it_errors_on_single_route_get_deployments_empty_zone_id() {
    let script_name = "single_route_empty_zone_id";
    let empty_zone_id = "";

    let test_toml = WranglerToml::zoned_single_route(script_name, empty_zone_id, PATTERN);
    let toml_string = toml::to_string(&test_toml).unwrap();
    let manifest = Manifest::from_str(&toml_string).unwrap();

    let environment = None;

    assert!(manifest.get_deployments(environment).is_err());
}

#[test]
fn it_errors_on_single_route_get_deployments_missing_zone_id() {
    let script_name = "single_route_empty_zone_id";
    let empty_zone_id = "";

    let mut test_toml = WranglerToml::zoned_single_route(script_name, empty_zone_id, PATTERN);
    test_toml.zone_id = None;
    let toml_string = toml::to_string(&test_toml).unwrap();
    let manifest = Manifest::from_str(&toml_string).unwrap();

    let environment = None;

    assert!(manifest.get_deployments(environment).is_err());
}

#[test]
fn it_errors_on_single_route_get_deployments_empty_route() {
    let script_name = "single_route_empty_route";
    let pattern = "";

    let test_toml = WranglerToml::zoned_single_route(script_name, ZONE_ID, pattern);
    let toml_string = toml::to_string(&test_toml).unwrap();
    let manifest = Manifest::from_str(&toml_string).unwrap();

    let environment = None;

    assert!(manifest.get_deployments(environment).is_err());
}

#[test]
fn it_errors_on_single_route_get_deployments_missing_route() {
    let script_name = "single_route_missing_route";

    let mut test_toml = WranglerToml::zoned_single_route(script_name, ZONE_ID, "");
    test_toml.route = None;
    let toml_string = toml::to_string(&test_toml).unwrap();
    let manifest = Manifest::from_str(&toml_string).unwrap();

    let environment = None;

    assert!(manifest.get_deployments(environment).is_err());
}

#[test]
fn it_can_get_a_multi_route_zoned_get_deployments() {
    let script_name = "multi_route_zoned";
    let patterns = [PATTERN, "blog.hostname.tld/*"];

    let mut test_toml = WranglerToml::webpack(script_name);
    test_toml.routes = Some(patterns.to_vec());
    test_toml.zone_id = Some(ZONE_ID);
    let toml_string = toml::to_string(&test_toml).unwrap();
    let manifest = Manifest::from_str(&toml_string).unwrap();

    let expected_routes = patterns
        .iter()
        .map(|p| Route {
            script: Some(script_name.to_string()),
            pattern: (*p).to_string(),
            id: None,
        })
        .collect();
    let expected_deployments = vec![DeployTarget::Zoned(ZonedTarget {
        zone_id: ZONE_ID.to_string(),
        routes: expected_routes,
    })];

    let environment = None;
    let actual_deployments = manifest.get_deployments(environment).unwrap();

    assert_eq!(actual_deployments, expected_deployments);
}

#[test]
fn it_can_get_a_multi_route_zoned_get_deployments_workers_dev_false() {
    let script_name = "multi_route_zoned_workers_dev_false";
    let patterns = [PATTERN, "blog.hostname.tld/*"];

    let mut test_toml = WranglerToml::webpack(script_name);
    test_toml.workers_dev = Some(false);
    test_toml.routes = Some(patterns.to_vec());
    test_toml.zone_id = Some(ZONE_ID);
    let toml_string = toml::to_string(&test_toml).unwrap();
    let manifest = Manifest::from_str(&toml_string).unwrap();

    let expected_routes = patterns
        .iter()
        .map(|p| Route {
            script: Some(script_name.to_string()),
            pattern: (*p).to_string(),
            id: None,
        })
        .collect();
    let expected_deployments = vec![DeployTarget::Zoned(ZonedTarget {
        zone_id: ZONE_ID.to_string(),
        routes: expected_routes,
    })];

    let environment = None;
    let actual_deployments = manifest.get_deployments(environment).unwrap();

    assert_eq!(actual_deployments, expected_deployments);
}

#[test]
fn it_can_get_multi_route_with_route() {
    let script_name = "multi_route_with_route";
    let patterns = [PATTERN];

    let mut test_toml = WranglerToml::webpack(script_name);
    test_toml.workers_dev = Some(false);
    test_toml.routes = Some(patterns.to_vec());
    test_toml.route = Some("blog.hostname.tld/*");
    test_toml.zone_id = Some(ZONE_ID);
    let toml_string = toml::to_string(&test_toml).unwrap();
    let manifest = Manifest::from_str(&toml_string).unwrap();

    let expected_routes = std::iter::once("blog.hostname.tld/*")
        .chain(patterns.iter().copied())
        .map(|p| Route {
            script: Some(script_name.to_string()),
            pattern: (*p).to_string(),
            id: None,
        })
        .collect();
    let expected_deployments = vec![DeployTarget::Zoned(ZonedTarget {
        zone_id: ZONE_ID.to_string(),
        routes: expected_routes,
    })];

    let environment = None;
    let actual_deployments = manifest.get_deployments(environment).unwrap();

    assert_eq!(actual_deployments, expected_deployments);
}

#[test]
fn it_errors_on_multi_route_get_deployments_empty_zone_id() {
    let script_name = "multi_route_empty_zone_id";
    let patterns = [PATTERN, "blog.hostname.tld/*"];
    let empty_zone_id = "";

    let test_toml = WranglerToml::zoned_multi_route(script_name, empty_zone_id, patterns.to_vec());
    let toml_string = toml::to_string(&test_toml).unwrap();
    let manifest = Manifest::from_str(&toml_string).unwrap();

    let environment = None;

    assert!(manifest.get_deployments(environment).is_err());
}

#[test]
fn it_errors_on_multi_route_get_deployments_missing_zone_id() {
    let script_name = "multi_route_missing_zone_id";
    let patterns = [PATTERN, "blog.hostname.tld/*"];
    let empty_zone_id = "";

    let mut test_toml =
        WranglerToml::zoned_multi_route(script_name, empty_zone_id, patterns.to_vec());
    test_toml.zone_id = None;
    let toml_string = toml::to_string(&test_toml).unwrap();
    let manifest = Manifest::from_str(&toml_string).unwrap();

    let environment = None;

    assert!(manifest.get_deployments(environment).is_err());
}

#[test]
fn it_errors_on_multi_route_get_deployments_empty_routes_list() {
    let script_name = "multi_route_empty_routes_list";
    let patterns = [];

    let test_toml = WranglerToml::zoned_multi_route(script_name, ZONE_ID, patterns.to_vec());
    let toml_string = toml::to_string(&test_toml).unwrap();
    let manifest = Manifest::from_str(&toml_string).unwrap();

    let environment = None;

    assert!(manifest.get_deployments(environment).is_err());
}

#[test]
fn it_errors_on_multi_route_get_deployments_empty_route() {
    let script_name = "multi_route_empty_route";
    let patterns = [""];

    let test_toml = WranglerToml::zoned_multi_route(script_name, ZONE_ID, patterns.to_vec());
    let toml_string = toml::to_string(&test_toml).unwrap();
    let manifest = Manifest::from_str(&toml_string).unwrap();

    let environment = None;

    assert!(manifest.get_deployments(environment).is_err());
}

#[test]
fn it_can_get_a_route_with_routes() {
    let script_name = "route_and_routes";
    let patterns = ["blog.hostname.tld/*"];

    let mut test_toml = WranglerToml::zoned_single_route(script_name, ZONE_ID, PATTERN);
    test_toml.routes = Some(patterns.to_vec());
    let toml_string = toml::to_string(&test_toml).unwrap();
    let manifest = Manifest::from_str(&toml_string).unwrap();

    let expected_routes = std::iter::once(PATTERN)
        .chain(patterns.iter().copied())
        .map(|p| Route {
            script: Some(script_name.to_string()),
            pattern: (*p).to_string(),
            id: None,
        })
        .collect();
    let expected_deployments = vec![DeployTarget::Zoned(ZonedTarget {
        routes: expected_routes,
        zone_id: ZONE_ID.to_owned(),
    })];

    let environment = None;
    let actual_deployments = manifest.get_deployments(environment).unwrap();

    assert_eq!(actual_deployments, expected_deployments);
}

#[test]
fn it_gets_deployments_with_route_and_workers_dev_true() {
    let script_name = "route_and_workers_dev";

    let mut test_toml = WranglerToml::zoned_single_route(script_name, ZONE_ID, PATTERN);
    test_toml.workers_dev = Some(true);
    let toml_string = toml::to_string(&test_toml).unwrap();
    let manifest = Manifest::from_str(&toml_string).unwrap();

    let environment = None;

    assert!(manifest.get_deployments(environment).is_err());
}

#[test]
fn it_gets_deployments_with_routes_and_workers_dev_true() {
    let script_name = "routes_and_workers_dev";
    let patterns = [PATTERN];

    let mut test_toml = WranglerToml::zoned_multi_route(script_name, ZONE_ID, patterns.to_vec());
    test_toml.workers_dev = Some(true);
    let toml_string = toml::to_string(&test_toml).unwrap();
    let manifest = Manifest::from_str(&toml_string).unwrap();

    let environment = None;

    assert!(manifest.get_deployments(environment).is_err());
}

// ENVIRONMENT TESTS
// Top level empty
#[test]
fn when_top_level_empty_env_empty() {
    let script_name = "top_level_empty_env_empty";
    let env_config = EnvConfig::default();

    let test_toml = WranglerToml::with_env(script_name, env_config);
    let toml_string = toml::to_string(&test_toml).unwrap();
    let manifest = Manifest::from_str(&toml_string).unwrap();

    let actual_deployments = manifest.get_deployments(Some(TEST_ENV_NAME));

    assert!(actual_deployments.is_err());
}

#[test]
fn when_top_level_empty_env_has_zone_id() {
    // if env only includes zone id, error
    let script_name = "when_top_level_empty_env_has_zone_id";
    let mut env_config = EnvConfig::default();
    env_config.zone_id = Some(ZONE_ID);

    let test_toml = WranglerToml::with_env(script_name, env_config);
    let toml_string = toml::to_string(&test_toml).unwrap();
    let manifest = Manifest::from_str(&toml_string).unwrap();

    let actual_deployments = manifest.get_deployments(Some(TEST_ENV_NAME));

    assert!(actual_deployments.is_err());
}

#[test]
fn when_top_level_empty_env_workers_dev_false() {
    let account_id = "testaccountid";
    let workers_dev = false;

    let env_config = EnvConfig::zoneless_with_account_id(workers_dev, account_id);

    let script_name = "top_level_empty_env_empty";
    let test_toml = WranglerToml::with_env(script_name, env_config);
    let toml_string = toml::to_string(&test_toml).unwrap();
    let manifest = Manifest::from_str(&toml_string).unwrap();

    let actual_deployments = manifest.get_deployments(Some(TEST_ENV_NAME));

    assert!(actual_deployments.is_err());
}

#[test]
fn when_top_level_empty_env_workers_dev_true() {
    let account_id = "testaccountid";
    let workers_dev = true;
    let env_config = EnvConfig::zoneless_with_account_id(workers_dev, account_id);

    let script_name = "top_level_empty_env_zoneless_true";
    let test_toml = WranglerToml::with_env(script_name, env_config);
    let toml_string = toml::to_string(&test_toml).unwrap();
    let manifest = Manifest::from_str(&toml_string).unwrap();

    let environment = Some(TEST_ENV_NAME);
    let actual_deployments = manifest.get_deployments(environment).unwrap();
    let expected_deployments = vec![DeployTarget::Zoneless(ZonelessTarget {
        script_name: manifest.worker_name(environment),
        account_id: account_id.to_string(),
    })];

    assert_eq!(actual_deployments, expected_deployments);
}

#[test]
fn when_top_level_empty_zoned_single_route_env() {
    // when route is empty, error
    let pattern = "";
    let env_config = EnvConfig::zoned_single_route(ZONE_ID, pattern);

    let script_name = "top_level_empty_env_zoned_single_route_empty";
    let test_toml = WranglerToml::with_env(script_name, env_config);
    let toml_string = toml::to_string(&test_toml).unwrap();
    let manifest = Manifest::from_str(&toml_string).unwrap();

    let actual_deployments = manifest.get_deployments(Some(TEST_ENV_NAME));

    assert!(actual_deployments.is_err());
}

#[test]
fn when_top_level_empty_env_zoned_single_route_no_zone_id() {
    let empty_zone_id = "";
    let env_config = EnvConfig::zoned_single_route(empty_zone_id, PATTERN);

    let script_name = "top_level_empty_env_zoned_single_route_no_zone_id";
    let test_toml = WranglerToml::with_env(script_name, env_config);
    let toml_string = toml::to_string(&test_toml).unwrap();
    let manifest = Manifest::from_str(&toml_string).unwrap();

    let actual_deployments = manifest.get_deployments(Some(TEST_ENV_NAME));

    assert!(actual_deployments.is_err());
}

#[test]
fn when_top_level_empty_env_zoned_single_route_zone_id_only() {
    let mut env_config = EnvConfig::default();
    env_config.zone_id = Some(ZONE_ID);

    let script_name = "top_level_empty_env_zoned_single_route_zone_id_only";
    let test_toml = WranglerToml::with_env(script_name, env_config);
    let toml_string = toml::to_string(&test_toml).unwrap();
    let manifest = Manifest::from_str(&toml_string).unwrap();

    let actual_deployments = manifest.get_deployments(Some(TEST_ENV_NAME));

    assert!(actual_deployments.is_err());
}

#[test]
fn when_top_level_empty_env_zoned_single_route() {
    let env_config = EnvConfig::zoned_single_route(ZONE_ID, PATTERN);

    let script_name = "top_level_empty_env_zoned_single_route_no_zone_id";
    let test_toml = WranglerToml::with_env(script_name, env_config);
    let toml_string = toml::to_string(&test_toml).unwrap();
    let manifest = Manifest::from_str(&toml_string).unwrap();

    let actual_deployments = manifest.get_deployments(Some(TEST_ENV_NAME)).unwrap();

    let expected_name = manifest.worker_name(Some(TEST_ENV_NAME));

    let expected_routes = vec![Route {
        script: Some(expected_name),
        pattern: PATTERN.to_string(),
        id: None,
    }];
    let expected_deployments = vec![DeployTarget::Zoned(ZonedTarget {
        zone_id: ZONE_ID.to_string(),
        routes: expected_routes,
    })];

    assert_eq!(actual_deployments, expected_deployments);
}

#[test]
fn when_top_level_empty_zoned_multi_route_env_routes_empty() {
    // when routes list is empty, error
    let patterns = [];
    let env_config = EnvConfig::zoned_multi_route(ZONE_ID, patterns.to_vec());

    let script_name = "top_level_empty_zoned_multi_route_env_routes_empty";
    let test_toml = WranglerToml::with_env(script_name, env_config);
    let toml_string = toml::to_string(&test_toml).unwrap();
    let manifest = Manifest::from_str(&toml_string).unwrap();

    let actual_deployments = manifest.get_deployments(Some(TEST_ENV_NAME));

    assert!(actual_deployments.is_err());
}

#[test]
fn when_top_level_empty_zoned_multi_route_env_route_empty() {
    // when route is empty, error
    let patterns = [""];
    let env_config = EnvConfig::zoned_multi_route(ZONE_ID, patterns.to_vec());

    let script_name = "top_level_empty_env_zoned_multi_route_empty";
    let test_toml = WranglerToml::with_env(script_name, env_config);
    let toml_string = toml::to_string(&test_toml).unwrap();
    let manifest = Manifest::from_str(&toml_string).unwrap();

    let actual_deployments = manifest.get_deployments(Some(TEST_ENV_NAME));

    assert!(actual_deployments.is_err());
}

#[test]
fn when_top_level_empty_zoned_multi_route_env_zone_id_missing() {
    // when zone id is missing, error
    let patterns = [PATTERN];
    let env_config = EnvConfig::zoned_multi_route("", patterns.to_vec());

    let script_name = "top_level_empty_zoned_multi_route_env_zone_id_missing";
    let test_toml = WranglerToml::with_env(script_name, env_config);
    let toml_string = toml::to_string(&test_toml).unwrap();
    let manifest = Manifest::from_str(&toml_string).unwrap();

    let actual_deployments = manifest.get_deployments(Some(TEST_ENV_NAME));

    assert!(actual_deployments.is_err());
}

#[test]
fn when_top_level_empty_zoned_multi_route_env() {
    // when zone id is present, all good
    let patterns = [PATTERN];
    let env_config = EnvConfig::zoned_multi_route(ZONE_ID, patterns.to_vec());

    let script_name = "top_level_empty_env_zoned_multi_route_no_zone_id";
    let test_toml = WranglerToml::with_env(script_name, env_config);
    let toml_string = toml::to_string(&test_toml).unwrap();
    let manifest = Manifest::from_str(&toml_string).unwrap();

    let actual_deployments = manifest.get_deployments(Some(TEST_ENV_NAME)).unwrap();

    let expected_name = manifest.worker_name(Some(TEST_ENV_NAME));

    let expected_routes = patterns
        .iter()
        .map(|p| Route {
            script: Some(expected_name.to_string()),
            pattern: (*p).to_string(),
            id: None,
        })
        .collect();

    let expected_deployments = vec![DeployTarget::Zoned(ZonedTarget {
        zone_id: ZONE_ID.to_string(),
        routes: expected_routes,
    })];

    assert_eq!(actual_deployments, expected_deployments);
}

#[test]
fn when_top_level_zoneless_env_empty() {
    let script_name = "top_level_zoneless_env_empty";
    let env_config = EnvConfig::default();
    let workers_dev = true;

    let test_toml =
        WranglerToml::zoneless_with_env(script_name, ACCOUNT_ID, workers_dev, env_config);
    let toml_string = toml::to_string(&test_toml).unwrap();
    let manifest = Manifest::from_str(&toml_string).unwrap();

    let environment = Some(TEST_ENV_NAME);
    let actual_deployments = manifest.get_deployments(environment).unwrap();
    let expected_deployments = vec![DeployTarget::Zoneless(ZonelessTarget {
        script_name: manifest.worker_name(environment),
        account_id: ACCOUNT_ID.to_string(),
    })];

    assert_eq!(actual_deployments, expected_deployments);
}

#[test]
fn when_top_level_zoneless_env_zoneless_workers_dev_false() {
    let env_config = EnvConfig::zoneless(false);

    let script_name = "top_level_zoneless_env_zoneless_workers_dev_false";
    let workers_dev = true;
    let test_toml =
        WranglerToml::zoneless_with_env(script_name, ACCOUNT_ID, workers_dev, env_config);
    let toml_string = toml::to_string(&test_toml).unwrap();
    let manifest = Manifest::from_str(&toml_string).unwrap();

    let actual_deployments = manifest.get_deployments(Some(TEST_ENV_NAME));

    assert!(actual_deployments.is_err());
}

#[test]
fn when_top_level_zoneless_env_zoneless_workers_dev_true() {
    // when env.workers_dev = true
    let env_config = EnvConfig::zoneless(true);

    let script_name = "top_level_zoneless_env_zoneless_workers_dev_false";
    let workers_dev = true;
    let test_toml =
        WranglerToml::zoneless_with_env(script_name, ACCOUNT_ID, workers_dev, env_config);
    let toml_string = toml::to_string(&test_toml).unwrap();
    let manifest = Manifest::from_str(&toml_string).unwrap();

    let environment = Some(TEST_ENV_NAME);
    let actual_deployments = manifest.get_deployments(environment).unwrap();
    let expected_deployments = vec![DeployTarget::Zoneless(ZonelessTarget {
        account_id: ACCOUNT_ID.to_string(),
        script_name: manifest.worker_name(environment),
    })];

    assert_eq!(actual_deployments, expected_deployments);
}

#[test]
fn when_top_level_zoneless_env_zoned_single_route_empty() {
    // when route is empty, error
    let pattern = "";
    let env_config = EnvConfig::zoned_single_route(ZONE_ID, pattern);

    let script_name = "top_level_zoneless_env_zoned_single_route_empty";
    let workers_dev = true;
    let test_toml =
        WranglerToml::zoneless_with_env(script_name, ACCOUNT_ID, workers_dev, env_config);
    let toml_string = toml::to_string(&test_toml).unwrap();
    let manifest = Manifest::from_str(&toml_string).unwrap();

    let environment = Some(TEST_ENV_NAME);
    let actual_deployments = manifest.get_deployments(environment).unwrap();
    let expected_deployments = vec![DeployTarget::Zoneless(ZonelessTarget {
        account_id: ACCOUNT_ID.to_string(),
        script_name: manifest.worker_name(environment),
    })];

    assert_eq!(actual_deployments, expected_deployments);
}

#[test]
fn when_top_level_zoneless_env_zoned_single_route_zone_id_missing() {
    let empty_zone_id = "";

    let env_config = EnvConfig::zoned_single_route(empty_zone_id, PATTERN);

    let script_name = "top_level_zoneless_env_zoned_single_route_zone_id_missing";
    let workers_dev = true;
    let test_toml =
        WranglerToml::zoneless_with_env(script_name, ACCOUNT_ID, workers_dev, env_config);
    let toml_string = toml::to_string(&test_toml).unwrap();
    let manifest = Manifest::from_str(&toml_string).unwrap();

    let actual_deployments = manifest.get_deployments(Some(TEST_ENV_NAME));

    assert!(actual_deployments.is_err());
}

#[test]
fn when_top_level_zoneless_env_zoned_single_route() {
    // when zone id is present, all good
    let env_config = EnvConfig::zoned_single_route(ZONE_ID, PATTERN);

    let script_name = "top_level_zoneless_env_zoned_single_route_no_zone_id";
    let workers_dev = true;
    let test_toml =
        WranglerToml::zoneless_with_env(script_name, ACCOUNT_ID, workers_dev, env_config);
    let toml_string = toml::to_string(&test_toml).unwrap();
    let manifest = Manifest::from_str(&toml_string).unwrap();

    let actual_deployments = manifest.get_deployments(Some(TEST_ENV_NAME)).unwrap();

    let expected_name = manifest.worker_name(Some(TEST_ENV_NAME));

    let expected_routes = vec![Route {
        script: Some(expected_name),
        pattern: PATTERN.to_string(),
        id: None,
    }];
    let expected_deployments = vec![DeployTarget::Zoned(ZonedTarget {
        zone_id: ZONE_ID.to_string(),
        routes: expected_routes,
    })];

    assert_eq!(actual_deployments, expected_deployments);
}

#[test]
fn when_top_level_zoneless_env_zoned_multi_route_routes_list_empty() {
    // when routes list is empty, error
    let patterns = [];
    let env_config = EnvConfig::zoned_multi_route(ZONE_ID, patterns.to_vec());

    let script_name = "top_level_zoneless_env_zoned_multi_route_routes_list_empty";
    let workers_dev = true;
    let test_toml =
        WranglerToml::zoneless_with_env(script_name, ACCOUNT_ID, workers_dev, env_config);
    let toml_string = toml::to_string(&test_toml).unwrap();
    let manifest = Manifest::from_str(&toml_string).unwrap();

    let actual_deployments = manifest.get_deployments(Some(TEST_ENV_NAME));

    assert!(actual_deployments.is_err());
}

#[test]
fn when_top_level_zoneless_env_zoned_multi_route_route_empty() {
    // when route is empty, error
    let patterns = [""];
    let env_config = EnvConfig::zoned_multi_route(ZONE_ID, patterns.to_vec());

    let script_name = "top_level_zoneless_env_zoned_multi_route_route_empty";
    let workers_dev = true;
    let test_toml =
        WranglerToml::zoneless_with_env(script_name, ACCOUNT_ID, workers_dev, env_config);
    let toml_string = toml::to_string(&test_toml).unwrap();
    let manifest = Manifest::from_str(&toml_string).unwrap();

    let actual_deployments = manifest.get_deployments(Some(TEST_ENV_NAME));

    assert!(actual_deployments.is_err());
}

#[test]
fn when_top_level_zoneless_env_zoned_multi_route_route_key_present() {
    // when route key also present, append routes to route
    let patterns = [PATTERN];
    let mut env_config = EnvConfig::zoned_multi_route(ZONE_ID, patterns.to_vec());
    env_config.route = Some("blog.hostname.tld/*");

    let script_name = "top_level_zoneless_env_zoned_multi_route_route_key_present";
    let workers_dev = true;
    let test_toml =
        WranglerToml::zoneless_with_env(script_name, ACCOUNT_ID, workers_dev, env_config);
    let toml_string = toml::to_string(&test_toml).unwrap();
    let manifest = Manifest::from_str(&toml_string).unwrap();

    let expected_name = manifest.worker_name(Some(TEST_ENV_NAME));
    let expected_routes = std::iter::once("blog.hostname.tld/*")
        .into_iter()
        .chain(patterns.iter().copied())
        .map(|p| Route {
            script: Some(expected_name.clone()),
            pattern: p.to_owned(),
            id: None,
        })
        .collect();

    let expected_deployments = vec![DeployTarget::Zoned(ZonedTarget {
        routes: expected_routes,
        zone_id: ZONE_ID.to_owned(),
    })];

    let actual_deployments = manifest.get_deployments(Some(TEST_ENV_NAME)).unwrap();

    assert_eq!(expected_deployments, actual_deployments);
}

#[test]
fn when_top_level_zoneless_env_zoned_multi_route_zone_id_missing() {
    // when zone id is missing, error
    let patterns = [PATTERN];
    let env_config = EnvConfig::zoned_multi_route("", patterns.to_vec());

    let script_name = "when_top_level_zoneless_env_zoned_multi_route_zone_id_missing";
    let workers_dev = true;
    let test_toml =
        WranglerToml::zoneless_with_env(script_name, ACCOUNT_ID, workers_dev, env_config);
    let toml_string = toml::to_string(&test_toml).unwrap();
    let manifest = Manifest::from_str(&toml_string).unwrap();

    let actual_deployments = manifest.get_deployments(Some(TEST_ENV_NAME));

    assert!(actual_deployments.is_err());
}

#[test]
fn when_top_level_zoneless_env_zoned_multi_route() {
    // when zone id is present, all good
    let patterns = [PATTERN];
    let env_config = EnvConfig::zoned_multi_route(ZONE_ID, patterns.to_vec());

    let script_name = "top_level_zoneless_env_zoned_multi_route_no_zone_id";
    let workers_dev = true;
    let test_toml =
        WranglerToml::zoneless_with_env(script_name, ACCOUNT_ID, workers_dev, env_config);
    let toml_string = toml::to_string(&test_toml).unwrap();
    let manifest = Manifest::from_str(&toml_string).unwrap();

    let actual_deployments = manifest.get_deployments(Some(TEST_ENV_NAME)).unwrap();

    let expected_name = manifest.worker_name(Some(TEST_ENV_NAME));

    let expected_routes = patterns
        .iter()
        .map(|p| Route {
            script: Some(expected_name.to_string()),
            pattern: (*p).to_string(),
            id: None,
        })
        .collect();

    let expected_deployments = vec![DeployTarget::Zoned(ZonedTarget {
        zone_id: ZONE_ID.to_string(),
        routes: expected_routes,
    })];

    assert_eq!(actual_deployments, expected_deployments);
}

#[test]
fn when_top_level_zoned_env_empty() {
    let env_config = EnvConfig::default();

    let script_name = "top_level_zoned_env_empty";
    let test_toml =
        WranglerToml::zoned_single_route_with_env(script_name, ZONE_ID, PATTERN, env_config);
    let toml_string = toml::to_string(&test_toml).unwrap();
    let manifest = Manifest::from_str(&toml_string).unwrap();

    let actual_deployments = manifest.get_deployments(Some(TEST_ENV_NAME));

    assert!(actual_deployments.is_err());
}

#[test]
fn when_top_level_zoned_env_zoneless_workers_dev_false() {
    // when env.workers_dev = false
    let workers_dev = false;
    let env_config = EnvConfig::zoneless_with_account_id(workers_dev, ACCOUNT_ID);

    let script_name = "top_level_zoned_env_zoneless_workers_dev_false";
    let test_toml =
        WranglerToml::zoned_single_route_with_env(script_name, ZONE_ID, PATTERN, env_config);
    let toml_string = toml::to_string(&test_toml).unwrap();
    let manifest = Manifest::from_str(&toml_string).unwrap();

    let actual_deployments = manifest.get_deployments(Some(TEST_ENV_NAME));

    assert!(actual_deployments.is_err());
}

#[test]
fn when_top_level_zoned_env_zoneless_workers_dev_true() {
    // when env.workers_dev = true
    let workers_dev = true;
    let env_config = EnvConfig::zoneless_with_account_id(workers_dev, ACCOUNT_ID);

    let script_name = "when_top_level_zoned_env_zoneless_workers_dev_true";
    let test_toml =
        WranglerToml::zoned_single_route_with_env(script_name, ZONE_ID, PATTERN, env_config);
    let toml_string = toml::to_string(&test_toml).unwrap();
    let manifest = Manifest::from_str(&toml_string).unwrap();

    let environment = Some(TEST_ENV_NAME);
    let actual_deployments = manifest.get_deployments(environment).unwrap();
    let expected_deployments = vec![DeployTarget::Zoneless(ZonelessTarget {
        account_id: ACCOUNT_ID.to_string(),
        script_name: manifest.worker_name(environment),
    })];

    assert_eq!(actual_deployments, expected_deployments);
}

#[test]
fn when_top_level_zoned_env_zoned_single_route_route_empty() {
    // when route is empty, error
    let env_pattern = "";
    let env_config = EnvConfig::zoned_single_route(ZONE_ID, env_pattern);

    let script_name = "top_level_zoned_env_zoned_single_route_empty";
    let test_toml =
        WranglerToml::zoned_single_route_with_env(script_name, ZONE_ID, PATTERN, env_config);
    let toml_string = toml::to_string(&test_toml).unwrap();
    let manifest = Manifest::from_str(&toml_string).unwrap();

    let actual_deployments = manifest.get_deployments(Some(TEST_ENV_NAME));

    assert!(actual_deployments.is_err());
}

#[test]
fn when_top_level_zoned_env_zoned_single_route_zone_id_missing() {
    // when zone id is missing, use top level zone
    let env_pattern = "env.hostname.tld/*";
    let env_config = EnvConfig::zoned_single_route("", env_pattern);

    let script_name = "top_level_zoned_env_zoned_single_route_no_zone_id";
    let test_toml =
        WranglerToml::zoned_single_route_with_env(script_name, ZONE_ID, PATTERN, env_config);
    let toml_string = toml::to_string(&test_toml).unwrap();
    let manifest = Manifest::from_str(&toml_string).unwrap();

    let actual_deployments = manifest.get_deployments(Some(TEST_ENV_NAME)).unwrap();

    let expected_name = manifest.worker_name(Some(TEST_ENV_NAME));

    let expected_routes = vec![Route {
        script: Some(expected_name),
        pattern: env_pattern.to_string(),
        id: None,
    }];
    let expected_deployments = vec![DeployTarget::Zoned(ZonedTarget {
        zone_id: ZONE_ID.to_string(),
        routes: expected_routes,
    })];

    assert_eq!(actual_deployments, expected_deployments);
}

#[test]
fn when_top_level_zoned_env_zoned_single_route() {
    // when zone id is present in env, use that
    let env_pattern = PATTERN;
    let env_zone_id = "sampleenvzoneid";
    let env_config = EnvConfig::zoned_single_route(env_zone_id, env_pattern);

    let script_name = "top_level_zoned_env_zoned_single_route_no_zone_id";
    let test_toml =
        WranglerToml::zoned_single_route_with_env(script_name, ZONE_ID, PATTERN, env_config);
    let toml_string = toml::to_string(&test_toml).unwrap();
    let manifest = Manifest::from_str(&toml_string).unwrap();

    let actual_deployments = manifest.get_deployments(Some(TEST_ENV_NAME)).unwrap();

    let expected_name = manifest.worker_name(Some(TEST_ENV_NAME));

    let expected_routes = vec![Route {
        script: Some(expected_name),
        pattern: PATTERN.to_string(),
        id: None,
    }];
    let expected_deployments = vec![DeployTarget::Zoned(ZonedTarget {
        zone_id: env_zone_id.to_string(),
        routes: expected_routes,
    })];

    assert_eq!(actual_deployments, expected_deployments);
}
