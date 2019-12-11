use std::collections::HashMap;

use serde::Serialize;

// small suite of flexible toml structs
// the  idea here is to focus on "when this config key is set"
// rather than needing to write tomls all the time.
// these structs set every value as an `Option`. To use,
// initialize a new WranglerToml::default() and begin setting
// values on it.
#[derive(Clone, Debug, Default, Serialize)]
pub struct KvConfig<'a> {
    pub binding: Option<&'a str>,
    pub id: Option<&'a str>,
}

#[derive(Clone, Debug, Default, Serialize)]
pub struct SiteConfig<'a> {
    pub bucket: Option<&'a str>,
    #[serde(rename = "entry-point")]
    pub entry_point: Option<&'a str>,
    pub include: Option<Vec<&'a str>>,
    pub exclude: Option<Vec<&'a str>>,
}

#[derive(Clone, Debug, Default, Serialize)]
pub struct EnvConfig<'a> {
    pub name: Option<&'a str>,
    pub account_id: Option<&'a str>,
    pub workers_dev: Option<bool>,
    pub route: Option<&'a str>,
    pub routes: Option<Vec<&'a str>>,
    pub zone_id: Option<&'a str>,
    pub webpack_config: Option<&'a str>,
    pub private: Option<bool>,
    pub site: Option<SiteConfig<'a>>,
    #[serde(rename = "kv-namespaces")]
    pub kv_namespaces: Option<Vec<KvConfig<'a>>>,
}

impl EnvConfig<'_> {
    pub fn custom_script_name(name: &str) -> EnvConfig {
        let mut env_config = EnvConfig::default();
        env_config.name = Some(name);

        eprintln!("{:#?}", &env_config);
        env_config
    }

    pub fn zoneless(is_workers_dev: bool) -> EnvConfig<'static> {
        let mut env_config = EnvConfig::default();
        env_config.workers_dev = Some(is_workers_dev);

        env_config
    }

    pub fn zoned_single_route<'a>(zone_id: &'a str, route: &'a str) -> EnvConfig<'a> {
        let mut env_config = EnvConfig::default();
        env_config.zone_id = Some(zone_id);
        env_config.route = Some(route);

        env_config
    }

    pub fn zoned_multi_route<'a>(zone_id: &'a str, routes: Vec<&'a str>) -> EnvConfig<'a> {
        let mut env_config = EnvConfig::default();
        env_config.zone_id = Some(zone_id);
        env_config.routes = Some(routes);

        env_config
    }
}

#[derive(Clone, Debug, Default, Serialize)]
pub struct WranglerToml<'a> {
    pub name: Option<&'a str>,
    #[serde(rename = "type")]
    pub target_type: Option<&'a str>,
    pub account_id: Option<&'a str>,
    pub workers_dev: Option<bool>,
    pub route: Option<&'a str>,
    pub routes: Option<Vec<&'a str>>,
    pub zone_id: Option<&'a str>,
    pub webpack_config: Option<&'a str>,
    pub private: Option<bool>,
    pub env: Option<HashMap<&'a str, EnvConfig<'a>>>,
    #[serde(rename = "kv-namespaces")]
    pub kv_namespaces: Option<Vec<KvConfig<'a>>>,
    pub site: Option<SiteConfig<'a>>,
}

impl WranglerToml<'_> {
    pub fn webpack(name: &str) -> WranglerToml {
        let mut wrangler_toml = WranglerToml::default();
        wrangler_toml.name = Some(name);
        wrangler_toml.target_type = Some("webpack");

        eprintln!("{:#?}", &wrangler_toml);
        wrangler_toml
    }

    pub fn webpack_zoneless(name: &str, is_workers_dev: bool) -> WranglerToml {
        let mut wrangler_toml = WranglerToml::webpack(name);
        wrangler_toml.workers_dev = Some(is_workers_dev);

        eprintln!("{:#?}", &wrangler_toml);
        wrangler_toml
    }

    pub fn webpack_zoned_single_route<'a>(
        name: &'a str,
        zone_id: &'a str,
        route: &'a str,
    ) -> WranglerToml<'a> {
        let mut wrangler_toml = WranglerToml::webpack(name);
        wrangler_toml.zone_id = Some(zone_id);
        wrangler_toml.route = Some(route);

        eprintln!("{:#?}", &wrangler_toml);
        wrangler_toml
    }

    pub fn webpack_zoned_multi_route<'a>(
        name: &'a str,
        zone_id: &'a str,
        routes: Vec<&'a str>,
    ) -> WranglerToml<'a> {
        let mut wrangler_toml = WranglerToml::webpack(name);
        wrangler_toml.zone_id = Some(zone_id);
        wrangler_toml.routes = Some(routes);

        eprintln!("{:#?}", &wrangler_toml);
        wrangler_toml
    }

    pub fn webpack_with_env<'a>(
        name: &'a str,
        env_name: &'a str,
        env_config: EnvConfig<'a>,
    ) -> WranglerToml<'a> {
        let mut wrangler_toml = WranglerToml::webpack(name);
        wrangler_toml.env = Some(test_env(env_name, env_config));

        eprintln!("{:#?}", &wrangler_toml);
        wrangler_toml
    }

    pub fn webpack_zoneless_with_env<'a>(
        name: &'a str,
        is_workers_dev: bool,
        env_name: &'a str,
        env_config: EnvConfig<'a>,
    ) -> WranglerToml<'a> {
        let mut wrangler_toml = WranglerToml::webpack_zoneless(name, is_workers_dev);
        wrangler_toml.env = Some(test_env(env_name, env_config));

        eprintln!("{:#?}", &wrangler_toml);
        wrangler_toml
    }

    pub fn webpack_zoned_single_route_with_env<'a>(
        name: &'a str,
        zone_id: &'a str,
        route: &'a str,
        env_name: &'a str,
        env_config: EnvConfig<'a>,
    ) -> WranglerToml<'a> {
        let mut wrangler_toml = WranglerToml::webpack_zoned_single_route(name, zone_id, route);
        wrangler_toml.env = Some(test_env(env_name, env_config));

        eprintln!("{:#?}", &wrangler_toml);
        wrangler_toml
    }

    pub fn webpack_zoned_multi_route_with_env<'a>(
        name: &'a str,
        zone_id: &'a str,
        routes: Vec<&'a str>,
        env_name: &'a str,
        env_config: EnvConfig<'a>,
    ) -> WranglerToml<'a> {
        let mut wrangler_toml = WranglerToml::webpack_zoned_multi_route(name, zone_id, routes);
        wrangler_toml.env = Some(test_env(env_name, env_config));

        eprintln!("{:#?}", &wrangler_toml);
        wrangler_toml
    }

    pub fn webpack_std_config(name: &str) -> WranglerToml {
        let mut wrangler_toml = WranglerToml::webpack_zoneless(name, true);
        wrangler_toml.webpack_config = Some("webpack.config.js");

        eprintln!("{:#?}", &wrangler_toml);
        wrangler_toml
    }

    pub fn webpack_custom_config<'a>(name: &'a str, webpack_config: &'a str) -> WranglerToml<'a> {
        let mut wrangler_toml = WranglerToml::webpack_zoneless(name, true);
        wrangler_toml.webpack_config = Some(webpack_config);

        eprintln!("{:#?}", &wrangler_toml);
        wrangler_toml
    }

    pub fn rust(name: &str) -> WranglerToml {
        let mut wrangler_toml = WranglerToml::default();
        wrangler_toml.name = Some(name);
        wrangler_toml.workers_dev = Some(true);
        wrangler_toml.target_type = Some("rust");

        eprintln!("{:#?}", &wrangler_toml);
        wrangler_toml
    }

    pub fn javascript(name: &str) -> WranglerToml {
        let mut wrangler_toml = WranglerToml::default();
        wrangler_toml.name = Some(name);
        wrangler_toml.workers_dev = Some(true);
        wrangler_toml.target_type = Some("javascript");

        eprintln!("{:#?}", &wrangler_toml);
        wrangler_toml
    }
}

fn test_env<'a>(env_name: &'a str, env_config: EnvConfig<'a>) -> HashMap<&'a str, EnvConfig<'a>> {
    let mut env = HashMap::new();
    env.insert(env_name, env_config);

    env
}
