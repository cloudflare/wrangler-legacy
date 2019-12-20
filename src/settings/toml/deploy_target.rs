use crate::settings::toml::Route;

impl DeployTarget {
    pub fn build(
        script_name: &str,
        route_config: &RouteConfig,
    ) -> Result<DeployTarget, failure::Error> {
        if route_config.workers_dev_false_by_itself() || route_config.has_conflicting_targets() {
            failure::bail!("you must set workers_dev = true or provide a zone_id and route/routes.")
        }

        if route_config.is_zoneless() {
            DeployTarget::build_zoneless(script_name, route_config)
        } else if route_config.is_zoned() {
            DeployTarget::build_zoned(script_name, route_config)
        } else {
            failure::bail!("No deploy target specified");
        }
    }

    fn build_zoneless(
        script_name: &str,
        route_config: &RouteConfig,
    ) -> Result<DeployTarget, failure::Error> {
        if let Some(account_id) = &route_config.account_id {
            // TODO: Deserialize empty strings to None
            if account_id.is_empty() {
                failure::bail!("field `account_id` is required to deploy to workers.dev");
            }
            let zoneless = Zoneless {
                script_name: script_name.to_string(),
                account_id: account_id.to_string(),
            };

            Ok(DeployTarget::Zoneless(zoneless))
        } else {
            failure::bail!("field `account_id` is required to deploy to workers.dev");
        }
    }

    fn build_zoned(
        script_name: &str,
        route_config: &RouteConfig,
    ) -> Result<DeployTarget, failure::Error> {
        if let Some(zone_id) = &route_config.zone_id {
            // TODO: Deserialize empty strings to None
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

            // TODO: these should be an if/else if block; write deserializer
            // for `route` key that turns `Some("")` into `None`
            if let Some(route) = &route_config.route {
                zoned.add_route(&route, script_name);
            }

            if let Some(routes) = &route_config.routes {
                for route in routes {
                    zoned.add_route(route, script_name);
                }
            }

            if zoned.routes.is_empty() {
                failure::bail!("No deploy target specified");
            }

            Ok(DeployTarget::Zoned(zoned))
        } else {
            failure::bail!("field `zone_id` is required to deploy to routes");
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum DeployTarget {
    Zoneless(Zoneless),
    Zoned(Zoned),
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

impl Zoned {
    pub fn add_route(&mut self, route: &str, script: &str) -> &Self {
        // TODO: Write custom deserializer for route, which will make this fn unnecessary
        if !route.is_empty() {
            self.routes.push(Route {
                id: None,
                script: Some(script.to_string()),
                pattern: route.to_string(),
            })
        }

        self
    }
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
    pub fn has_conflicting_targets(&self) -> bool {
        if self.is_zoneless() {
            self.has_routes_defined()
        } else if let Some(routes) = &self.routes {
            !routes.is_empty() && self.route.is_some()
        } else {
            false
        }
    }

    pub fn has_routes_defined(&self) -> bool {
        if self.route.is_some() {
            true
        } else if let Some(routes) = &self.routes {
            !routes.is_empty()
        } else {
            false
        }
    }

    pub fn is_zoneless(&self) -> bool {
        self.workers_dev.unwrap_or_default()
    }

    pub fn is_zoned(&self) -> bool {
        self.has_routes_defined() || self.zone_id.is_some()
    }

    pub fn workers_dev_false_by_itself(&self) -> bool {
        if let Some(workers_dev) = self.workers_dev {
            !workers_dev && !self.has_routes_defined()
        } else {
            false
        }
    }
}
