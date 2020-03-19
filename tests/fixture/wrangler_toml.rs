#![allow(dead_code)]
use std::collections::HashMap;

use serde::Serialize;

pub const TEST_ENV_NAME: &str = "test";

// small suite of flexible toml structs
// the  idea here is to focus on "when this config key is set"
// rather than needing to write tomls all the time.
// these structs set every value as an `Option`. To use,
// initialize a new WranglerToml::default() and begin setting
// values on it.
#[derive(Clone, Debug, Default, Serialize)]
pub struct KvConfig {
  pub binding: Option<&'static str>,
  pub id: Option<&'static str>,
}

#[derive(Clone, Debug, Default, Serialize)]
pub struct SiteConfig {
  pub bucket: Option<&'static str>,
  #[serde(rename = "entry-point")]
  pub entry_point: Option<&'static str>,
  pub include: Option<Vec<&'static str>>,
  pub exclude: Option<Vec<&'static str>>,
}

#[derive(Clone, Debug, Default, Serialize)]
pub struct EnvConfig {
  pub name: Option<&'static str>,
  pub account_id: Option<&'static str>,
  pub workers_dev: Option<bool>,
  pub route: Option<&'static str>,
  pub routes: Option<Vec<&'static str>>,
  pub zone_id: Option<&'static str>,
  pub webpack_config: Option<&'static str>,
  pub private: Option<bool>,
  pub site: Option<SiteConfig>,
  #[serde(rename = "kv-namespaces")]
  pub kv_namespaces: Option<Vec<KvConfig>>,
  pub vars: Option<HashMap<&'static str, &'static str>>,
}

impl EnvConfig {
  pub fn custom_script_name(name: &'static str) -> EnvConfig {
    let mut env_config = EnvConfig::default();
    env_config.name = Some(name);

    env_config
  }

  pub fn zoneless(workers_dev: bool) -> EnvConfig {
    let mut env_config = EnvConfig::default();
    env_config.workers_dev = Some(workers_dev);

    env_config
  }

  pub fn zoneless_with_account_id(workers_dev: bool, account_id: &'static str) -> EnvConfig {
    let mut env_config = EnvConfig::default();
    env_config.account_id = Some(account_id);
    env_config.workers_dev = Some(workers_dev);

    env_config
  }

  pub fn zoned_single_route(zone_id: &'static str, route: &'static str) -> EnvConfig {
    let mut env_config = EnvConfig::default();
    env_config.zone_id = Some(zone_id);
    env_config.route = Some(route);

    env_config
  }

  pub fn zoned_multi_route(zone_id: &'static str, routes: Vec<&'static str>) -> EnvConfig {
    let mut env_config = EnvConfig::default();
    env_config.zone_id = Some(zone_id);
    env_config.routes = Some(routes);

    env_config
  }
}

#[derive(Clone, Debug, Default, Serialize)]
pub struct WranglerToml {
  pub name: Option<&'static str>,
  #[serde(rename = "type")]
  pub target_type: Option<&'static str>,
  pub account_id: Option<&'static str>,
  pub workers_dev: Option<bool>,
  pub route: Option<&'static str>,
  pub routes: Option<Vec<&'static str>>,
  pub zone_id: Option<&'static str>,
  pub webpack_config: Option<&'static str>,
  pub private: Option<bool>,
  pub env: Option<HashMap<&'static str, EnvConfig>>,
  #[serde(rename = "kv-namespaces")]
  pub kv_namespaces: Option<Vec<KvConfig>>,
  pub site: Option<SiteConfig>,
  pub vars: Option<HashMap<&'static str, &'static str>>,
}

impl WranglerToml {
  // base build configs
  pub fn webpack(name: &'static str) -> WranglerToml {
    let mut wrangler_toml = WranglerToml::default();
    wrangler_toml.name = Some(name);
    wrangler_toml.target_type = Some("webpack");

    wrangler_toml
  }

  pub fn zoneless(name: &'static str, account_id: &'static str, workers_dev: bool) -> WranglerToml {
    let mut wrangler_toml = WranglerToml::webpack(name);
    wrangler_toml.workers_dev = Some(workers_dev);
    wrangler_toml.account_id = Some(account_id);

    wrangler_toml
  }

  pub fn zoned_single_route(
    name: &'static str,
    zone_id: &'static str,
    route: &'static str,
  ) -> WranglerToml {
    let mut wrangler_toml = WranglerToml::webpack(name);
    wrangler_toml.zone_id = Some(zone_id);
    wrangler_toml.route = Some(route);

    eprintln!("{:#?}", &wrangler_toml);
    wrangler_toml
  }

  pub fn zoned_multi_route(
    name: &'static str,
    zone_id: &'static str,
    routes: Vec<&'static str>,
  ) -> WranglerToml {
    let mut wrangler_toml = WranglerToml::webpack(name);
    wrangler_toml.zone_id = Some(zone_id);
    wrangler_toml.routes = Some(routes);

    eprintln!("{:#?}", &wrangler_toml);
    wrangler_toml
  }

  pub fn with_env(name: &'static str, env_config: EnvConfig) -> WranglerToml {
    let mut wrangler_toml = WranglerToml::webpack(name);
    wrangler_toml.env = Some(test_env(env_config));

    eprintln!("{:#?}", &wrangler_toml);
    wrangler_toml
  }

  pub fn zoneless_with_env(
    name: &'static str,
    account_id: &'static str,
    workers_dev: bool,
    env_config: EnvConfig,
  ) -> WranglerToml {
    let mut wrangler_toml = WranglerToml::zoneless(name, account_id, workers_dev);
    wrangler_toml.env = Some(test_env(env_config));

    eprintln!("{:#?}", &wrangler_toml);
    wrangler_toml
  }

  pub fn zoned_single_route_with_env(
    name: &'static str,
    zone_id: &'static str,
    route: &'static str,
    env_config: EnvConfig,
  ) -> WranglerToml {
    let mut wrangler_toml = WranglerToml::zoned_single_route(name, zone_id, route);
    wrangler_toml.env = Some(test_env(env_config));

    eprintln!("{:#?}", &wrangler_toml);
    wrangler_toml
  }

  pub fn webpack_build(name: &'static str) -> WranglerToml {
    let mut wrangler_toml = WranglerToml::default();
    wrangler_toml.name = Some(name);
    wrangler_toml.workers_dev = Some(true);
    wrangler_toml.target_type = Some("webpack");

    wrangler_toml
  }

  pub fn webpack_std_config(name: &'static str) -> WranglerToml {
    let mut wrangler_toml = WranglerToml::webpack_build(name);
    wrangler_toml.webpack_config = Some("webpack.config.js");

    wrangler_toml
  }

  pub fn webpack_custom_config(name: &'static str, webpack_config: &'static str) -> WranglerToml {
    let mut wrangler_toml = WranglerToml::webpack_build(name);
    wrangler_toml.webpack_config = Some(webpack_config);

    wrangler_toml
  }

  pub fn rust(name: &'static str) -> WranglerToml {
    let mut wrangler_toml = WranglerToml::default();
    wrangler_toml.name = Some(name);
    wrangler_toml.workers_dev = Some(true);
    wrangler_toml.target_type = Some("rust");

    wrangler_toml
  }

  pub fn javascript(name: &'static str) -> WranglerToml {
    let mut wrangler_toml = WranglerToml::default();
    wrangler_toml.name = Some(name);
    wrangler_toml.workers_dev = Some(true);
    wrangler_toml.target_type = Some("javascript");

    wrangler_toml
  }

  pub fn site(name: &'static str) -> WranglerToml {
    let mut wrangler_toml = WranglerToml::webpack_build(name);
    let mut site = SiteConfig::default();
    site.bucket = Some("./public");
    wrangler_toml.site = Some(site);

    wrangler_toml
  }
}

fn test_env(env_config: EnvConfig) -> HashMap<&'static str, EnvConfig> {
  let mut env = HashMap::new();
  env.insert(TEST_ENV_NAME, env_config);

  env
}
