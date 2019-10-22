pub mod kv_namespace;
mod target_type;

pub use kv_namespace::KvNamespace;
pub use target_type::TargetType;

use std::collections::{HashMap, HashSet};
use std::env;
use std::fs;
use std::path::{Path, PathBuf};

use config::{Config, File};
use serde::{Deserialize, Serialize};

use crate::terminal::emoji;
use crate::terminal::message;

const SITE_ENTRY_POINT: &str = "workers-site";

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Site {
    pub bucket: String,
    #[serde(rename = "entry-point")]
    pub entry_point: Option<String>,
    pub include: Option<Vec<String>>,
    pub exclude: Option<Vec<String>>,
}

impl Site {
    pub fn new(bucket: &str) -> Site {
        let mut site = Site::default();
        site.bucket = String::from(bucket);

        site
    }
}

impl Default for Site {
    fn default() -> Site {
        Site {
            bucket: String::new(),
            entry_point: Some(String::from(SITE_ENTRY_POINT)),
            include: None,
            exclude: None,
        }
    }
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct TemplateConfig {
    pub account_id: Option<String>,
    pub env: Option<HashMap<String, Environment>>,
    #[serde(rename = "kv-namespaces")]
    pub kv_namespaces: Option<Vec<KvNamespace>>,
    #[serde(rename = "type")]
    pub target_type: TargetType,
    pub route: Option<String>,
    pub routes: Option<HashMap<String, String>>,
    pub webpack_config: Option<String>,
    pub workers_dev: Option<bool>,
    pub zone_id: Option<String>,
    pub site: Option<Site>,
}

impl TemplateConfig {
    pub fn warn_on_account_info(&self) {
        let mut top_level_fields: Vec<String> = Vec::new();
        if let Some(account_id) = &self.account_id {
            if !account_id.is_empty() {
                top_level_fields.push("account_id".to_string());
            }
        }
        if let Some(kv_namespaces) = &self.kv_namespaces {
            for kv_namespace in kv_namespaces {
                if !kv_namespace.id.is_empty() && !kv_namespace.binding.is_empty() {
                    top_level_fields.push(format!("kv-namespace {}", kv_namespace.binding));
                }
            }
        }
        if let Some(route) = &self.route {
            if !route.is_empty() {
                top_level_fields.push("route".to_string());
            }
        }
        if let Some(zone_id) = &self.zone_id {
            if !zone_id.is_empty() {
                top_level_fields.push("zone_id".to_string());
            }
        }

        let mut env_fields: HashMap<String, Vec<String>> = HashMap::new();

        if let Some(env) = &self.env {
            for (env_name, env) in env {
                let mut current_env_fields: Vec<String> = Vec::new();
                if let Some(account_id) = &env.account_id {
                    if !account_id.is_empty() {
                        current_env_fields.push("account_id".to_string());
                    }
                }
                if let Some(kv_namespaces) = &env.kv_namespaces {
                    for kv_namespace in kv_namespaces {
                        if !kv_namespace.id.is_empty() && !kv_namespace.binding.is_empty() {
                            current_env_fields
                                .push(format!("kv-namespace {}", kv_namespace.binding));
                        }
                    }
                }
                if let Some(route) = &env.route {
                    if !route.is_empty() {
                        current_env_fields.push("route".to_string());
                    }
                }
                if let Some(zone_id) = &env.zone_id {
                    if !zone_id.is_empty() {
                        current_env_fields.push("zone_id".to_string());
                    }
                }
                if !current_env_fields.is_empty() {
                    env_fields.insert(env_name.to_string(), current_env_fields);
                }
            }
        }
        let top_level_separator = "\n- ";
        let env_separator = "\n  - ";
        if !top_level_fields.is_empty() || !env_fields.is_empty() {
            message::warn("Replace all account specific info in your wrangler.toml.");
            message::warn(
                "Your zone_id and account_id can be found in the right sidebar at https://dash.cloudflare.com",
            );
            if !top_level_fields.is_empty() {
                let top_level_fields = top_level_fields
                    .clone()
                    .into_iter()
                    .collect::<Vec<String>>()
                    .join(top_level_separator);
                println!("{}{}", top_level_separator, top_level_fields);
            }
            if !env_fields.is_empty() {
                for (env_name, env_fields) in env_fields {
                    let msg_prefix = format!("[env.{}]", env_name);
                    let env_fields = env_fields
                        .clone()
                        .into_iter()
                        .collect::<Vec<String>>()
                        .join(env_separator);
                    println!("{}{}{}", msg_prefix, env_separator, env_fields);
                }
            }
        }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Target {
    pub account_id: String,
    #[serde(rename = "kv-namespaces")]
    pub kv_namespaces: Option<Vec<KvNamespace>>,
    pub name: String,
    #[serde(rename = "type")]
    pub target_type: TargetType,
    pub route: Option<String>,
    pub routes: Option<HashMap<String, String>>,
    pub webpack_config: Option<String>,
    pub zone_id: Option<String>,
    pub site: Option<Site>,
}

impl Target {
    pub fn kv_namespaces(&self) -> Vec<KvNamespace> {
        self.kv_namespaces.clone().unwrap_or_else(Vec::new)
    }

    pub fn add_kv_namespace(&mut self, kv_namespace: KvNamespace) {
        let mut updated_namespaces = self.kv_namespaces();
        updated_namespaces.push(kv_namespace);
        self.kv_namespaces = Some(updated_namespaces);
    }

    pub fn build_dir(&self) -> Result<PathBuf, std::io::Error> {
        let current_dir = env::current_dir()?;
        // if `site` is configured, we want to isolate worker code
        // and build artifacts away from static site application code.
        // if the user has configured `site.entry-point`, use that
        // as the build directory. Otherwise use the default const
        // SITE_BUILD_DIR
        match &self.site {
            Some(site_config) => Ok(current_dir.join(
                site_config
                    .entry_point
                    .to_owned()
                    .unwrap_or_else(|| format!("./{}", SITE_ENTRY_POINT)),
            )),
            None => Ok(current_dir),
        }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Environment {
    pub account_id: Option<String>,
    pub name: Option<String>,
    pub private: Option<bool>,
    pub route: Option<String>,
    pub routes: Option<HashMap<String, String>>,
    pub webpack_config: Option<String>,
    pub workers_dev: Option<bool>,
    pub zone_id: Option<String>,
    pub site: Option<Site>,
    #[serde(rename = "kv-namespaces")]
    pub kv_namespaces: Option<Vec<KvNamespace>>,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct Manifest {
    pub account_id: String,
    pub name: String,
    pub private: Option<bool>,
    #[serde(rename = "type")]
    pub target_type: TargetType,
    pub route: Option<String>,
    pub routes: Option<HashMap<String, String>>,
    pub webpack_config: Option<String>,
    pub workers_dev: Option<bool>,
    pub zone_id: Option<String>,
    pub site: Option<Site>,
    #[serde(rename = "kv-namespaces")]
    pub kv_namespaces: Option<Vec<KvNamespace>>,
    pub env: Option<HashMap<String, Environment>>,
}

impl Manifest {
    pub fn new(config_path: &Path) -> Result<Self, failure::Error> {
        let config = read_config(config_path)?;

        // check for pre 1.1.0 KV namespace format
        let kv_namespaces: Result<Vec<config::Value>, config::ConfigError> =
            config.get("kv-namespaces");

        validate_kv_namespaces_config(kv_namespaces)?;

        let manifest: Manifest = config.try_into()?;

        check_for_duplicate_names(&manifest)?;

        Ok(manifest)
    }

    pub fn generate(
        name: String,
        target_type: Option<TargetType>,
        config_path: &PathBuf,
        site: Option<Site>,
    ) -> Result<Manifest, failure::Error> {
        let config_file = config_path.join("wrangler.toml");
        let template_config_content = fs::read_to_string(&config_file);
        let template_config = match &template_config_content {
            Ok(content) => {
                let config: TemplateConfig = toml::from_str(content)?;
                config.warn_on_account_info();
                if let Some(target_type) = &target_type {
                    if config.target_type != *target_type {
                        message::warn(&format!("The template recommends the \"{}\" type. Using type \"{}\" may cause errors, we recommend changing the type field in wrangler.toml to \"{}\"", config.target_type, target_type, config.target_type));
                    }
                }
                Ok(config)
            }
            Err(err) => Err(err),
        };
        let template_config = match template_config {
            Ok(config) => config,
            Err(err) => {
                log::info!("Error parsing template {}", err);
                log::debug!("template content {:?}", template_config_content);
                TemplateConfig::default()
            }
        };
        // TODO: https://github.com/cloudflare/wrangler/issues/773
        let manifest = Manifest {
            account_id: template_config
                .account_id
                .clone()
                .unwrap_or_else(|| String::new()),
            name: name.clone(),
            private: None,
            target_type: target_type.unwrap_or_else(|| template_config.clone().target_type),
            route: template_config.route.or_else(|| Some(String::new())),
            routes: None,
            webpack_config: template_config.webpack_config,
            workers_dev: template_config.workers_dev.or_else(|| Some(true)),
            zone_id: template_config.zone_id.or_else(|| Some(String::new())),
            site: template_config.site.or(site),
            kv_namespaces: template_config.kv_namespaces.clone(),
            env: template_config.env.clone(),
        };

        let toml = toml::to_string(&manifest)?;

        log::info!("Writing a wrangler.toml file at {}", config_file.display());
        fs::write(&config_file, &toml)?;
        Ok(manifest)
    }

    pub fn get_target(&self, environment_name: Option<&str>) -> Result<Target, failure::Error> {
        // Site projects are always Webpack for now; don't let toml override this.
        let target_type = match self.site {
            Some(_) => TargetType::Webpack,
            None => self.target_type.clone(),
        };

        let mut target = Target {
            target_type,                                 // MUST inherit
            account_id: self.account_id.clone(),         // MAY inherit
            webpack_config: self.webpack_config.clone(), // MAY inherit
            zone_id: self.zone_id.clone(),               // MAY inherit
            // importantly, the top level name will be modified
            // to include the name of the environment
            name: self.name.clone(),                   // MAY inherit
            kv_namespaces: self.kv_namespaces.clone(), // MUST NOT inherit
            route: None, // can inherit None, but not Some (see negotiate_zoneless)
            routes: self.routes.clone(), // MUST NOT inherit
            site: self.site.clone(), // MUST NOT inherit
        };

        let environment = self.get_environment(environment_name)?;

        self.check_private(environment);

        target.route = self.negotiate_zoneless(environment)?;
        if let Some(environment) = environment {
            target.name = if let Some(name) = &environment.name {
                name.clone()
            } else {
                match environment_name {
                    Some(environment_name) => format!("{}-{}", self.name, environment_name),
                    None => failure::bail!("You must specify `name` in your wrangler.toml"),
                }
            };
            if let Some(account_id) = &environment.account_id {
                target.account_id = account_id.clone();
            }
            if environment.routes.is_some() {
                target.routes = environment.routes.clone();
            }
            if environment.webpack_config.is_some() {
                target.webpack_config = environment.webpack_config.clone();
            }
            if environment.zone_id.is_some() {
                target.zone_id = environment.zone_id.clone();
            }
            // don't inherit kv namespaces because it is an anti-pattern to use the same namespaces across multiple environments
            target.kv_namespaces = environment.kv_namespaces.clone();
        }

        Ok(target)
    }

    fn get_environment(
        &self,
        environment_name: Option<&str>,
    ) -> Result<Option<&Environment>, failure::Error> {
        // check for user-specified environment name
        if let Some(environment_name) = environment_name {
            if let Some(environment_table) = &self.env {
                if let Some(environment) = environment_table.get(environment_name) {
                    Ok(Some(environment))
                } else {
                    failure::bail!(format!(
                        "{} Could not find environment with name \"{}\"",
                        emoji::WARN,
                        environment_name
                    ))
                }
            } else {
                failure::bail!(format!(
                    "{} There are no environments specified in your wrangler.toml",
                    emoji::WARN
                ))
            }
        } else {
            Ok(None)
        }
    }

    // this function takes the workers_dev booleans and the routes in a manifest
    // and then returns an Option<String> representing the deploy target
    // if it is None, it means deploy to workers.dev, otherwise deploy to the route

    // no environments:
    // +-------------+---------------------+------------------------------+
    // | workers_dev |        route        |            result            |
    // +-------------+---------------------+------------------------------+
    // | None        | None                | failure: pick target         |
    // | None        | Some("")            | failure: pick target         |
    // | None        | Some("example.com") | Some("example.com")          |
    // | false       | None                | failure: pick target         |
    // | false       | Some("")            | failure: pick target         |
    // | false       | Some("example.com") | Some("example.com")          |
    // | true        | None                | None                         |
    // | true        | Some("")            | None                         |
    // | true        | Some("example.com") | failure: conflicting targets |
    // +-------------+---------------------+------------------------------+
    //
    // When environments are introduced, this truth table holds true with workers_dev being inherited
    // and route being ignored.
    // if top level workers_dev is true, it is inherited but can be overridden by an env route
    //
    // this will fail with empty_route_failure
    // workers_dev = true
    // [env.foo]
    // route = ""
    //
    // this will return Some("example.com")
    // workers_dev = true
    // [env.foo]
    // route = "example.com"
    fn negotiate_zoneless(
        &self,
        environment: Option<&Environment>,
    ) -> Result<Option<String>, failure::Error> {
        let conflicting_targets_failure = "Your environment should only include `workers_dev` or `route`. If you are trying to publish to workers.dev, remove `route` from your wrangler.toml, if you are trying to publish to your own domain, remove `workers_dev`.";
        let pick_target_failure =
            "You must specify either `workers_dev` or `route` and `zone_id` in order to publish.";
        let empty_route_failure =
            "If you want to deploy to workers.dev, remove `route` from your environment config.";

        log::debug!("top level workers_dev: {:?}", self.workers_dev);
        log::debug!("top level route: {:?}", self.route);

        // start with top level configuration
        let (top_workers_dev, top_route) = match (self.workers_dev, self.route.clone()) {
            (None, Some(route)) => (false, Some(route)),
            (Some(workers_dev), None) => (workers_dev, None),
            (Some(workers_dev), Some(route)) => (workers_dev, Some(route)),
            (None, None) => (false, None),
        };

        // override top level with environment
        let (workers_dev, route) = if let Some(env) = &environment {
            log::debug!("env workers_dev: {:?}", env.workers_dev);
            log::debug!("env route: {:?}", env.route);
            match (env.workers_dev, env.route.clone()) {
                (None, Some(route)) => {
                    if top_workers_dev && route.is_empty() {
                        failure::bail!(empty_route_failure)
                    } else {
                        (false, Some(route))
                    }
                }
                (Some(workers_dev), None) => (workers_dev, None),
                (Some(workers_dev), Some(route)) => {
                    if route.is_empty() && workers_dev {
                        failure::bail!(empty_route_failure)
                    }
                    (workers_dev, Some(route))
                }
                (None, None) => (top_workers_dev, top_route),
            }
        } else {
            (top_workers_dev, top_route)
        };

        log::debug!("negotiated workers_dev: {}", workers_dev);
        log::debug!("negotiated route: {:?}", route);

        match (workers_dev, route) {
            (true, None) => Ok(None),
            (true, Some(route)) => {
                if route.is_empty() {
                    Ok(None)
                } else {
                    failure::bail!(conflicting_targets_failure)
                }
            }
            (false, Some(route)) => {
                if route.is_empty() {
                    failure::bail!(pick_target_failure)
                } else {
                    Ok(Some(route))
                }
            }
            (false, None) => failure::bail!(pick_target_failure),
        }
    }

    fn check_private(&self, environment: Option<&Environment>) {
        let deprecate_private_warning = "The `private` field is deprecated; please use \
        `workers_dev` to toggle between publishing to your workers.dev subdomain and your own domain.";

        // Check for the presence of the 'private' field in top-level config; if present, warn.
        if self.private.is_some() {
            message::warn(deprecate_private_warning);
        }

        // Also check for presence of 'private' field in a provided environment; if present, warn
        if let Some(e) = environment {
            if e.private.is_some() {
                message::warn(deprecate_private_warning);
            }
        }
    }
}

fn read_config(config_path: &Path) -> Result<Config, failure::Error> {
    let mut config = Config::new();

    let config_str = config_path
        .to_str()
        .expect("project config path should be a string");
    config.merge(File::with_name(config_str))?;

    // Eg.. `CF_ACCOUNT_AUTH_KEY=farts` would set the `account_auth_key` key
    config.merge(config::Environment::with_prefix("CF"))?;

    Ok(config)
}

fn validate_kv_namespaces_config(
    kv_namespaces: Result<Vec<config::Value>, config::ConfigError>,
) -> Result<(), failure::Error> {
    if let Ok(values) = kv_namespaces {
        let old_format = values.iter().any(|val| val.clone().into_str().is_ok());

        if old_format {
            message::warn("As of 1.1.0 the kv-namespaces format has been stabilized");
            message::info("Please add a section like this in your `wrangler.toml` for each KV Namespace you wish to bind:");

            let fmt_demo = r##"
[[kv-namespaces]]
binding = "BINDING_NAME"
id = "0f2ac74b498b48028cb68387c421e279"

# binding is the variable name you wish to bind the namespace to in your script.
# id is the namespace_id assigned to your kv namespace upon creation. e.g. (per namespace)
"##;

            println!("{}", fmt_demo);

            let msg = format!("{0} Your project config has an error {0}", emoji::WARN);
            failure::bail!(msg)
        }
    }
    Ok(())
}

fn check_for_duplicate_names(manifest: &Manifest) -> Result<(), failure::Error> {
    let mut names: HashSet<String> = HashSet::new();
    let mut duplicate_names: HashSet<String> = HashSet::new();
    names.insert(manifest.name.to_string());
    if let Some(environments) = &manifest.env {
        for (_, environment) in environments.iter() {
            if let Some(name) = &environment.name {
                if names.contains(name) && !duplicate_names.contains(name) {
                    duplicate_names.insert(name.to_string());
                } else {
                    names.insert(name.to_string());
                }
            }
        }
    }
    let duplicate_name_string = duplicate_names
        .clone()
        .into_iter()
        .collect::<Vec<String>>()
        .join(", ");
    let duplicate_message = match duplicate_names.len() {
        1 => Some("this name is duplicated".to_string()),
        n if n >= 2 => Some("these names are duplicated".to_string()),
        _ => None,
    };
    if let Some(message) = duplicate_message {
        failure::bail!(format!(
            "{} Each name in your `wrangler.toml` must be unique, {}: {}",
            emoji::WARN,
            message,
            duplicate_name_string
        ))
    }
    Ok(())
}

#[cfg(test)]
mod tests;
