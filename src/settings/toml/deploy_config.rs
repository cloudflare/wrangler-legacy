use crate::settings::toml::{ConfigActorNamespaceImpl, Route};
use crate::terminal::message::{Message, StdOut};

#[derive(Clone, Debug, PartialEq)]
pub struct DeployConfig {
    pub actor_namespaces: Option<ActorNamespacesDeployConfig>,
    pub http_routes: HttpRouteDeployConfig,
    // It is an arbitrary limitation of wrangler
    // to only allow configuring either routes on a single zone, or a workers.dev subdomain.
    //
    // With the addition of new invocation types that make sense alongside the existing
    // route-based invocations, this limitation is more painful. So, I've replaced DeployConfig
    // with a struct type that can allow for multiple invocations per environment.
    // Invocation was chosen as the name as to not overload the word event, as actor invocations
    // and http route invocations share a protocol (http) and an event type (fetch).
    //
    // However, this single-deploy assumption is baked into the user interface and implementation
    // of wrangler dev, which automatically detects the hostname to use from your deploy config,
    // and uses the detected hostname to determine which api path to hit for zoned vs zoneless.
    //
    // Someday, RouteDeployConfig should be eliminated, and folded into DeployConfig
    // with the following fields placed directly in DeployConfig:
    // zoneless_invocation: Option<ZonelessConfig>
    // zoned_invocation: Option<ZonedConfig>
}

impl DeployConfig {
    pub fn build(
        script_name: &str,
        invocation_config: &InvocationConfig,
    ) -> Result<DeployConfig, failure::Error> {
        let http_routes = HttpRouteDeployConfig::build(script_name, invocation_config)?;
        let actor_namespaces = ActorNamespacesDeployConfig::build(invocation_config)?;
        Ok(DeployConfig {
            actor_namespaces,
            http_routes,
        })
    }
}

impl From<HttpRouteDeployConfig> for DeployConfig {
    fn from(http_routes: HttpRouteDeployConfig) -> DeployConfig {
        DeployConfig {
            actor_namespaces: None,
            http_routes,
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum HttpRouteDeployConfig {
    Zoneless(Zoneless),
    Zoned(Zoned),
    NoRoutes,
}

impl HttpRouteDeployConfig {
    pub fn build(
        script_name: &str,
        invocation_config: &InvocationConfig,
    ) -> Result<HttpRouteDeployConfig, failure::Error> {
        if invocation_config.has_conflicting_targets() {
            failure::bail!(
                "You cannot set workers_dev = true AND provide a zone_id and route/routes, you must pick one."
            )
        }

        invocation_config.warn_if_no_routes();

        if invocation_config.is_zoneless() {
            HttpRouteDeployConfig::build_zoneless(script_name, invocation_config)
        } else if invocation_config.maybe_zoned() {
            HttpRouteDeployConfig::build_zoned(script_name, invocation_config)
        } else {
            Ok(HttpRouteDeployConfig::NoRoutes)
        }
    }

    fn build_zoneless(
        script_name: &str,
        invocation_config: &InvocationConfig,
    ) -> Result<HttpRouteDeployConfig, failure::Error> {
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

            Ok(HttpRouteDeployConfig::Zoneless(zoneless))
        } else {
            failure::bail!("field `account_id` is required to deploy to workers.dev");
        }
    }

    fn build_zoned(
        script_name: &str,
        invocation_config: &InvocationConfig,
    ) -> Result<HttpRouteDeployConfig, failure::Error> {
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
                Ok(HttpRouteDeployConfig::NoRoutes)
            } else {
                Ok(HttpRouteDeployConfig::Zoned(zoned))
            }
        } else {
            failure::bail!("field `zone_id` is required to deploy to routes");
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct ActorNamespacesDeployConfig {
    pub account_id: String,
    pub actor_namespaces: Vec<ConfigActorNamespaceImpl>,
}

impl ActorNamespacesDeployConfig {
    pub fn build(
        invocation_config: &InvocationConfig,
    ) -> Result<Option<ActorNamespacesDeployConfig>, failure::Error> {
        if let Some(actor_namespaces) = &invocation_config.actor_namespaces {
            if actor_namespaces.is_empty() {
                StdOut::warn("your configuration file contains an empty list for actor namespaces to be implemented");
                return Ok(None);
            }
            if let Some(account_id) = &invocation_config.account_id {
                Ok(Some(ActorNamespacesDeployConfig {
                    account_id: account_id.clone(),
                    actor_namespaces: actor_namespaces.clone(),
                }))
            } else {
                failure::bail!("field `account_id` is required to deploy actor namespaces");
            }
        } else {
            Ok(None)
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
    pub actor_namespaces: Option<Vec<ConfigActorNamespaceImpl>>,
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
