use crate::settings::toml::Route;

#[derive(Debug)]
pub struct RouteConfig {
    pub workers_dev: Option<bool>,
    pub route: Option<String>,
    pub routes: Option<Vec<String>>,
    pub zone_id: Option<String>,
    pub account_id: Option<String>,
}

impl RouteConfig {
    fn has_conflicting_targets(&self) -> bool {
        if self.workers_dev.unwrap_or_default() {
            self.routes_defined()
        } else {
            if let Some(pattern) = &self.route {
                !pattern.is_empty() && self.routes.is_some()
            } else {
                false
            }
        }
    }

    pub fn routes_defined(&self) -> bool {
        if let Some(pattern) = &self.route {
            // this is all so messy because of deserializer
            !pattern.is_empty() || self.routes.is_some()
        } else {
            self.routes.is_some()
        }
    }

    pub fn is_zoneless(&self) -> bool {
        self.workers_dev.unwrap_or_default() && !self.has_conflicting_targets()
    }

    pub fn is_zoned(&self) -> bool {
        self.routes_defined() || !self.missing_zone_id()
    }

    // zone id is another weird one where `Some("")` is treated the same as `None`
    pub fn missing_zone_id(&self) -> bool {
        if let Some(zone_id) = &self.zone_id {
            zone_id.is_empty()
        } else {
            true
        }
    }

    // account id is another weird one where `Some("")` is treated the same as `None`
    pub fn missing_account_id(&self) -> bool {
        if let Some(account_id) = &self.account_id {
            account_id.is_empty()
        } else {
            true
        }
    }

    pub fn workers_dev_false_by_itself(&self) -> bool {
        if let Some(workers_dev) = self.workers_dev {
            !workers_dev && !self.routes_defined()
        } else {
            false
        }
    }
}

impl DeployTarget {
    pub fn build(
        script_name: &String,
        route_config: &RouteConfig,
    ) -> Result<DeployTarget, failure::Error> {
        if route_config.is_zoneless() {
            // account_id is required
            let account_id = route_config.account_id.as_ref().unwrap();
            if account_id.is_empty() {
                failure::bail!("field `account_id` is required to deploy to workers.dev");
            }
            let zoneless = Zoneless {
                script_name: script_name.to_string(),
                account_id: account_id.to_string(),
            };

            Ok(DeployTarget::Zoneless(zoneless))
        } else {
            // zone_id is required
            let zone_id = route_config.zone_id.as_ref().unwrap();
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
            if let Some(pattern) = &route_config.route {
                zoned.add_route(&pattern, script_name);
            }

            if let Some(patterns) = &route_config.routes {
                for pattern in patterns {
                    zoned.add_route(&pattern, script_name);
                }
            }

            if zoned.routes.is_empty() {
                failure::bail!("No deploy target specified");
            }

            Ok(DeployTarget::Zoned(zoned))
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
    pub fn add_route(&mut self, pattern: &String, script: &String) -> &Self {
        // TODO: Write custom deserializer for route, which will make this fn unnecessary
        if !pattern.is_empty() {
            self.routes.push(Route {
                id: None,
                script: Some(script.to_string()),
                pattern: pattern.to_string(),
            })
        }

        self
    }
}
