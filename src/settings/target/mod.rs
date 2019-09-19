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
}

impl Default for Site {
    fn default() -> Site {
        Site {
            bucket: String::new(),
            entry_point: Some(String::from(SITE_ENTRY_POINT)),
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
    pub workers_dev: bool,
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
                    .unwrap_or_else(|| format!("./{}}", SITE_ENTRY_POINT)),
            )),
            None => Ok(current_dir),
        }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Environment {
    pub account_id: Option<String>,
    #[serde(rename = "kv-namespaces")]
    pub kv_namespaces: Option<Vec<KvNamespace>>,
    pub name: Option<String>,
    pub private: Option<bool>,
    pub route: Option<String>,
    pub routes: Option<HashMap<String, String>>,
    pub webpack_config: Option<String>,
    pub workers_dev: Option<bool>,
    pub zone_id: Option<String>,
    pub site: Option<Site>,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct Manifest {
    pub account_id: String,
    pub env: Option<HashMap<String, Environment>>,
    #[serde(rename = "kv-namespaces")]
    pub kv_namespaces: Option<Vec<KvNamespace>>,
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
        target_type: TargetType,
        config_path: PathBuf,
        site: bool,
    ) -> Result<Manifest, failure::Error> {
        let mut manifest = Manifest::default();

        manifest.name = name.clone();
        manifest.target_type = target_type.clone();

        manifest.route = Some(String::new());
        manifest.workers_dev = Some(true);
        manifest.zone_id = Some(String::new());

        if site {
            let site = Site::default();
            manifest.site = Some(site);
        }

        let toml = toml::to_string(&manifest)?;
        let config_file = config_path.join("wrangler.toml");

        log::info!("Writing a wrangler.toml file at {}", config_file.display());
        fs::write(&config_file, &toml)?;
        Ok(manifest)
    }

    pub fn get_target(
        &self,
        environment_name: Option<&str>,
        release: bool,
    ) -> Result<Target, failure::Error> {
        if release && self.workers_dev.is_some() {
            failure::bail!(format!(
                "{} The --release flag is not compatible with use of the workers_dev field.",
                emoji::WARN
            ))
        }

        if release {
            message::warn("--release will be deprecated.");
        }

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
            workers_dev: true,                           // MAY inherit
            // importantly, the top level name will be modified
            // to include the name of the environment
            name: self.name.clone(),                   // MAY inherit
            kv_namespaces: self.kv_namespaces.clone(), // MUST NOT inherit
            route: None,                               // MUST NOT inherit
            routes: self.routes.clone(),               // MUST NOT inherit
            site: self.site.clone(),                   // MUST NOT inherit
        };

        let environment = self.get_environment(environment_name)?;

        self.check_private(environment);

        let (route, workers_dev) = self.negotiate_zoneless(environment, release)?;
        target.route = route;
        target.workers_dev = workers_dev;
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

    // TODO: when --release is deprecated, this will be much easier
    fn negotiate_zoneless(
        &self,
        environment: Option<&Environment>,
        release: bool,
    ) -> Result<(Option<String>, bool), failure::Error> {
        let use_dot_dev_failure =
            "Please specify the workers_dev boolean in the top level of your wrangler.toml.";
        let use_dot_dev_warning =
            format!("{}\n{} If you do not add workers_dev, this command may act unexpectedly in v1.5.0. Please see https://github.com/cloudflare/wrangler/blob/master/docs/content/environments.md for more information.", use_dot_dev_failure, emoji::WARN);
        let wdd_failure = format!(
            "{} Your environment should only include `workers_dev` or `route`. If you are trying to publish to workers.dev, remove `route` from your wrangler.toml, if you are trying to publish to your own domain, remove `workers_dev`.",
            emoji::WARN
        );

        // TODO: deprecate --release, remove warnings and parsing
        // switch wrangler publish behavior to act the same at top level
        // and environments
        // brace yourself, this is hairy
        let workers_dev = match environment {
            // top level configuration
            None => {
                if release {
                    match self.workers_dev {
                        Some(_) => {
                            failure::bail!(format!("{} {}", emoji::WARN, use_dot_dev_failure))
                        }
                        None => {
                            message::warn(&use_dot_dev_warning);
                            false // wrangler publish --release w/o workers_dev is zoned deploy
                        }
                    }
                } else if let Some(wdd) = self.workers_dev {
                    if wdd {
                        if let Some(route) = &self.route {
                            if !route.is_empty() {
                                failure::bail!(wdd_failure)
                            }
                        }
                    }
                    wdd
                } else {
                    message::warn(&use_dot_dev_warning);
                    true // wrangler publish w/o workers_dev is zoneless deploy
                }
            }

            // environment configuration
            Some(environment) => {
                if let Some(wdd) = environment.workers_dev {
                    if wdd && environment.route.is_some() {
                        failure::bail!(wdd_failure)
                    }
                    wdd
                } else if let Some(wdd) = self.workers_dev {
                    if wdd && environment.route.is_some() {
                        false // allow route to override workers_dev = true if wdd is inherited
                    } else {
                        wdd // inherit from top level
                    }
                } else {
                    false // if absent -> false
                }
            }
        };

        let route = if let Some(environment) = environment {
            if let Some(route) = &environment.route {
                if let Some(wdd) = environment.workers_dev {
                    if wdd {
                        failure::bail!(wdd_failure);
                    }
                }
                Some(route.clone())
            } else {
                None
            }
        } else {
            self.route.clone()
        };

        Ok((route, workers_dev))
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
