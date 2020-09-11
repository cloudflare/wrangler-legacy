use crate::settings::toml::Route;
use crate::terminal::message::{Message, StdOut};
#[derive(Clone, Debug, PartialEq)]
pub enum DeployConfig {
    Zoneless(Zoneless),
    Zoned(Zoned),
}

impl DeployConfig {
    pub fn build(
        script_name: &str,
        route_config: &RouteConfig,
    ) -> Result<DeployConfig, failure::Error> {
        if route_config.is_valid() {
            failure::bail!(
                "you must set EITHER workers_dev = true OR provide a zone_id and route/routes."
            )
        }

        if route_config.is_zoneless() {
            DeployConfig::build_zoneless(script_name, route_config)
        } else if route_config.is_zoned() {
            DeployConfig::build_zoned(script_name, route_config)
        } else {
            failure::bail!("No deploy target specified");
        }
    }

    fn build_zoneless(
        script_name: &str,
        route_config: &RouteConfig,
    ) -> Result<DeployConfig, failure::Error> {
        if let Some(account_id) = &route_config.account_id {
            // TODO: Deserialize empty strings to None; cannot do this for account id
            // yet without a large refactor.
            if account_id.is_empty() {
                failure::bail!("field `account_id` is required to deploy to workers.dev");
            }
            let zoneless = Zoneless {
                script_name: script_name.to_string(),
                account_id: account_id.to_string(),
            };

            Ok(DeployConfig::Zoneless(zoneless))
        } else {
            failure::bail!("field `account_id` is required to deploy to workers.dev");
        }
    }

    fn build_zoned(
        script_name: &str,
        route_config: &RouteConfig,
    ) -> Result<DeployConfig, failure::Error> {
        if let Some(zone_id) = &route_config.zone_id {
            if zone_id.is_empty() {
                failure::bail!("field `zone_id` is required to deploy to routes");
            }

            if route_config.has_conflicting_targets() {
                failure::bail!("specify either `route` or `routes`");
            }

            let mut zoned = Zoned {
                zone_id: zone_id.to_owned(),
                routes: Vec::new(),
            };

            if let Some(route) = &route_config.route {
                zoned.routes.push(Route {
                    id: None,
                    script: Some(script_name.to_string()),
                    pattern: route.to_string(),
                });
            } else if let Some(routes) = &route_config.routes {
                for route in routes {
                    if route.is_empty() {
                        StdOut::warn("your configuration file contains an empty route")
                    } else {
                        zoned.routes.push(Route {
                            id: None,
                            script: Some(script_name.to_string()),
                            pattern: route.to_string(),
                        })
                    }
                }
            }

            if zoned.routes.is_empty() {
                failure::bail!("No routes specified");
            }

            Ok(DeployConfig::Zoned(zoned))
        } else {
            failure::bail!("field `zone_id` is required to deploy to routes");
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Zoneless {
    pub account_id: String,
    pub script_name: String,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Zoned {
    pub zone_id: String,
    pub routes: Vec<Route>,
}

#[derive(Debug)]
pub struct RouteConfig {
    pub workers_dev: Option<bool>,
    pub route: Option<String>,
    pub routes: Option<Vec<String>>,
    pub zone_id: Option<String>,
    pub account_id: Option<String>,
}

impl RouteConfig {
    fn is_valid(&self) -> bool {
        self.workers_dev_false_by_itself() || self.has_conflicting_targets()
    }

    fn has_conflicting_targets(&self) -> bool {
        if self.is_zoneless() {
            self.has_routes_defined()
        } else if let Some(routes) = &self.routes {
            !routes.is_empty() && self.route.is_some()
        } else {
            false
        }
    }

    fn has_routes_defined(&self) -> bool {
        if self.route.is_some() {
            true
        } else if let Some(routes) = &self.routes {
            !routes.is_empty()
        } else {
            false
        }
    }

    fn is_zoneless(&self) -> bool {
        self.workers_dev.unwrap_or_default()
    }

    fn is_zoned(&self) -> bool {
        self.has_routes_defined() || self.zone_id.is_some()
    }

    fn workers_dev_false_by_itself(&self) -> bool {
        if let Some(workers_dev) = self.workers_dev {
            !workers_dev && !self.has_routes_defined()
        } else {
            false
        }
    }
}
