use crate::settings::toml::Route;
use crate::terminal::message::{Message, StdOut};
#[derive(Clone, Debug, PartialEq)]
pub enum DeployConfig {
    Zoneless(Zoneless),
    Zoned(Zoned),
    NoRoutes,
}

impl DeployConfig {
    pub fn build(
        script_name: &str,
        invocation_config: &InvocationConfig,
    ) -> Result<DeployConfig, failure::Error> {
        if invocation_config.has_conflicting_targets() {
            failure::bail!(
                "You cannot set workers_dev = true AND provide a zone_id and route/routes, you must pick one."
            )
        }

        invocation_config.warn_if_no_routes();

        if invocation_config.is_zoneless() {
            DeployConfig::build_zoneless(script_name, invocation_config)
        } else if invocation_config.maybe_zoned() {
            DeployConfig::build_zoned(script_name, invocation_config)
        } else {
            Ok(DeployConfig::NoRoutes)
        }
    }

    fn build_zoneless(
        script_name: &str,
        invocation_config: &InvocationConfig,
    ) -> Result<DeployConfig, failure::Error> {
        if let Some(account_id) = &invocation_config.account_id {
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
        invocation_config: &InvocationConfig,
    ) -> Result<DeployConfig, failure::Error> {
        if let Some(zone_id) = &invocation_config.zone_id {
            if zone_id.is_empty() {
                failure::bail!("field `zone_id` is required to deploy to routes");
            }

            if invocation_config.has_route_and_routes() {
                failure::bail!("specify either `route` or `routes`");
            }

            let mut zoned = Zoned {
                zone_id: zone_id.to_owned(),
                routes: Vec::new(),
            };

            if let Some(route) = &invocation_config.route {
                zoned.routes.push(Route {
                    id: None,
                    script: Some(script_name.to_string()),
                    pattern: route.to_string(),
                });
            } else if let Some(routes) = &invocation_config.routes {
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
                Ok(DeployConfig::NoRoutes)
            } else {
                Ok(DeployConfig::Zoned(zoned))
            }
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
pub struct InvocationConfig {
    pub workers_dev: Option<bool>,
    pub route: Option<String>,
    pub routes: Option<Vec<String>>,
    pub zone_id: Option<String>,
    pub account_id: Option<String>,
}

impl InvocationConfig {
    fn has_conflicting_targets(&self) -> bool {
        [
            self.is_zoneless() && self.maybe_zoned(),
            self.has_route_and_routes(),
        ]
        .iter()
        .any(|b| *b)
    }

    fn has_route_and_routes(&self) -> bool {
        self.route.is_some() && self.has_routes_in_routes_vec()
    }

    fn has_routes_defined(&self) -> bool {
        [self.route.is_some(), self.has_routes_in_routes_vec()]
            .iter()
            .any(|b| *b)
    }

    fn has_routes_in_routes_vec(&self) -> bool {
        self.routes.as_ref().map_or(false, |r| !r.is_empty())
    }

    fn is_zoneless(&self) -> bool {
        self.workers_dev.unwrap_or_default()
    }

    fn maybe_zoned(&self) -> bool {
        self.has_routes_defined() || self.zone_id.is_some()
    }

    fn warn_if_no_routes(&self) {
        if !self.has_routes_defined() && !self.is_zoneless() {
            let no_routes_hint = if self.zone_id.is_some() {
                "You have a zone_id configured, but no routes configured"
            } else {
                "You have not configured a zone_id and routes, or set workers_dev = true"
            };
            StdOut::warn(
                format!(
                    "{}. Your worker will still be uploaded, but no routing changes will be made.",
                    no_routes_hint
                )
                .as_str(),
            );
        }
    }
}
