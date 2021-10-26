use std::collections::{HashMap, HashSet};
use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::str::FromStr;

use config::{Config, File};

use anyhow::{anyhow, Result};
use chrono::Utc;
use once_cell::sync::OnceCell;
use serde::{Deserialize, Serialize};
use serde_with::rust::string_empty_as_none;

use super::migrations::{MigrationConfig, MigrationTag, Migrations};
use super::UsageModel;
use crate::commands::whoami::fetch_accounts;
use crate::commands::{validate_worker_name, whoami, DEFAULT_CONFIG_PATH};
use crate::deploy::{self, DeployTarget, DeploymentSet};
use crate::settings::global_user::GlobalUser;
use crate::settings::toml::builder::Builder;
use crate::settings::toml::dev::Dev;
use crate::settings::toml::durable_objects::DurableObjects;
use crate::settings::toml::environment::Environment;
use crate::settings::toml::kv_namespace::{ConfigKvNamespace, KvNamespace};
use crate::settings::toml::r2_bucket::{ConfigR2Bucket, R2Bucket};
use crate::settings::toml::route::RouteConfig;
use crate::settings::toml::site::Site;
use crate::settings::toml::target_type::TargetType;
use crate::settings::toml::triggers::Triggers;
use crate::settings::toml::Target;
use crate::terminal::{
    emoji,
    message::{Message, StdOut},
    styles,
};

#[derive(Clone, Debug, Default, Deserialize, PartialEq, Serialize)]
pub struct Manifest {
    #[serde(default)]
    pub name: String,
    #[serde(rename = "type")]
    pub target_type: TargetType,
    #[serde(default)]
    pub account_id: LazyAccountId,
    pub workers_dev: Option<bool>,
    #[serde(default, with = "string_empty_as_none")]
    pub route: Option<String>,
    pub routes: Option<Vec<String>>,
    #[serde(default, with = "string_empty_as_none")]
    pub zone_id: Option<String>,
    pub webpack_config: Option<String>,
    pub build: Option<Builder>,
    pub private: Option<bool>,
    pub dev: Option<Dev>,
    pub triggers: Option<Triggers>,
    pub migrations: Option<Vec<MigrationConfig>>,
    #[serde(default, with = "string_empty_as_none")]
    pub usage_model: Option<UsageModel>,
    pub compatibility_date: Option<String>,
    #[serde(default)]
    pub compatibility_flags: Vec<String>,
    pub durable_objects: Option<DurableObjects>,
    pub env: Option<HashMap<String, Environment>>,
    #[serde(alias = "kv-namespaces")]
    pub kv_namespaces: Option<Vec<ConfigKvNamespace>>,
    pub r2_buckets: Option<Vec<ConfigR2Bucket>>,
    // TODO: maybe one day, serde toml support will allow us to serialize sites
    // as a TOML inline table (this would prevent confusion with environments too!)
    pub site: Option<Site>,
    pub vars: Option<HashMap<String, String>>,
    pub text_blobs: Option<HashMap<String, PathBuf>>,
    pub wasm_modules: Option<HashMap<String, PathBuf>>,
}

impl Manifest {
    pub fn new(config_path: &Path) -> Result<Self> {
        let file_name = config_path.file_name().unwrap().to_str().unwrap();
        let mut message = format!("{} not found", file_name);
        if config_path.to_str().unwrap() == DEFAULT_CONFIG_PATH {
            message.push_str("; run `wrangler init` to create one.");
        }
        anyhow::ensure!(config_path.exists(), message);
        let config = read_config(config_path)?;

        let manifest: Manifest = match config.try_into() {
            Ok(m) => m,
            Err(e) => {
                if e.to_string().contains("unknown field `kv-namespaces`") {
                    anyhow::bail!("kv-namespaces should not live under the [site] table in your configuration file; please move it above [site].")
                } else {
                    anyhow::bail!(e)
                }
            }
        };

        check_for_duplicate_names(&manifest)?;

        Ok(manifest)
    }

    pub fn generate(
        name: String,
        target_type: Option<TargetType>,
        config_path: &Path,
        site: Option<Site>,
    ) -> Result<Manifest> {
        let config_file = &config_path.join("wrangler.toml");
        let config_template_str = fs::read_to_string(config_file).unwrap_or_else(|err| {
            log::info!("Error reading config template: {}", err);
            log::info!("Using default instead");
            toml::to_string_pretty(&Manifest::default())
                .expect("serializing the default toml should never fail")
        });

        let config_template =
            toml::from_str::<Manifest>(&config_template_str).unwrap_or_else(|err| {
                log::info!("Error parsing config template: {}", err);
                log::info!("Using default instead");
                Manifest::default()
            });

        config_template.warn_on_account_info();

        let default_workers_dev = match &config_template.route {
            Some(route) if route.is_empty() => Some(true),
            None => Some(true),
            _ => None,
        };

        /*
         * We use toml-edit for actually changing the template provided wrangler.toml,
         * since toml-edit is a format-preserving parser. Elsewhere, we use only toml-rs,
         * as only toml-rs supports serde.
         */

        let mut config_template_doc = config_template_str
            .parse::<toml_edit::Document>()
            .map_err(|err| anyhow!("toml_edit failed to parse config template. {}", err))?;

        config_template_doc["name"] = toml_edit::value(name);
        if let Some(default_workers_dev) = default_workers_dev {
            config_template_doc["workers_dev"] = toml_edit::value(default_workers_dev);
        }
        if let Some(target_type) = &target_type {
            if target_type.to_string() == "rust" {
                config_template_doc["type"] = toml_edit::value(TargetType::JavaScript.to_string());
            } else {
                config_template_doc["type"] = toml_edit::value(target_type.to_string());
            }
        }
        if let Some(site) = site {
            if config_template.site.is_none() {
                config_template_doc["site"]["bucket"] =
                    toml_edit::value(site.bucket.to_string_lossy().as_ref());

                if let Some(entry_point) = &site.entry_point {
                    config_template_doc["site"]["entry-point"] =
                        toml_edit::value(entry_point.to_string_lossy().as_ref());
                }
                if let Some(include) = &site.include {
                    let mut arr = toml_edit::Array::default();
                    include.iter().for_each(|i| {
                        arr.push(i.as_ref()).unwrap();
                    });
                    config_template_doc["site"]["include"] = toml_edit::value(arr);
                }
                if let Some(exclude) = &site.exclude {
                    let mut arr = toml_edit::Array::default();
                    exclude.iter().for_each(|i| {
                        arr.push(i.as_ref()).unwrap();
                    });
                    config_template_doc["site"]["exclude"] = toml_edit::value(arr);
                }
            }
        }

        config_template_doc["compatibility_date"] =
            toml_edit::value(Utc::now().format("%F").to_string());

        // TODO: https://github.com/cloudflare/wrangler/issues/773

        let toml = config_template_doc.to_string_in_original_order();
        let manifest = toml::from_str::<Manifest>(&toml)?;

        log::info!("Writing a wrangler.toml file at {}", config_file.display());
        fs::write(&config_file, &toml)?;
        Ok(manifest)
    }

    pub fn worker_name(&self, env_arg: Option<&str>) -> String {
        if let Some(environment) = self.get_environment(env_arg).unwrap_or_default() {
            if let Some(name) = &environment.name {
                return name.clone();
            }
            if let Some(env) = env_arg {
                return format!("{}-{}", self.name, env);
            }
        }

        self.name.clone()
    }

    fn route_config(&self) -> RouteConfig {
        RouteConfig {
            account_id: self.account_id.clone(),
            workers_dev: self.workers_dev,
            route: self.route.clone(),
            routes: self.routes.clone(),
            zone_id: self.zone_id.clone(),
        }
    }

    pub fn get_deployments(&self, env: Option<&str>) -> Result<DeploymentSet> {
        let script = self.worker_name(env);
        validate_worker_name(&script)?;

        let mut deployments = DeploymentSet::new();

        let env = self.get_environment(env)?;

        let mut add_routed_deployments = |route_config: &RouteConfig| -> Result<()> {
            if route_config.is_zoned() {
                let zoned = deploy::ZonedTarget::build(&script, route_config)?;

                if zoned.routes.is_empty() {
                    return Ok(());
                }

                // This checks all of the configured routes for the wildcard ending and warns
                // the user that their site may not work as expected without it.
                if self.site.is_some() {
                    let no_star_routes = zoned
                        .routes
                        .iter()
                        .filter(|r| !r.pattern.ends_with('*'))
                        .map(|r| r.pattern.as_str())
                        .collect::<Vec<_>>();
                    if !no_star_routes.is_empty() {
                        StdOut::warn(&format!(
                            "The following routes in your configuration file should have a trailing * to apply the Worker on every path, otherwise your site will not behave as expected.\n{}",
                            no_star_routes.join("\n"))
                        );
                    }
                }

                deployments.push(DeployTarget::Zoned(zoned));
            }

            if route_config.is_zoneless() {
                let zoneless = deploy::ZonelessTarget::build(&script, route_config)?;
                deployments.push(DeployTarget::Zoneless(zoneless));
            }

            if !route_config.is_zoned()
                && !route_config.is_zoneless()
                && (route_config.route.is_some()
                    || (route_config.routes.is_some()
                        && !route_config.routes.as_ref().unwrap().is_empty()))
            {
                anyhow::bail!(
                    "Routes specified with no zone, specify `zone_id` in your wrangler.toml"
                )
            }

            Ok(())
        };

        if let Some(env) = env {
            if let Some(env_route_cfg) = env.route_config(
                self.account_id.if_present().cloned(),
                self.zone_id.clone(),
                self.workers_dev,
            ) {
                add_routed_deployments(&env_route_cfg)
            } else {
                let config = self.route_config();
                if config.is_zoned() {
                    anyhow::bail!("you must specify route(s) per environment for zoned deploys.");
                } else {
                    add_routed_deployments(&config)
                }
            }
        } else {
            add_routed_deployments(&self.route_config())
        }?;

        let crons = match env {
            Some(e) => {
                let account_id = match e.account_id.as_ref() {
                    Some(id) => id,
                    None => self.account_id.load()?,
                };
                e.triggers
                    .as_ref()
                    .or_else(|| self.triggers.as_ref())
                    .map(|t| (t.crons.as_slice(), account_id))
            }
            None => match self.triggers.as_ref() {
                None => None,
                Some(t) => Some((t.crons.as_slice(), self.account_id.load()?)),
            },
        };

        if let Some((crons, account)) = crons {
            let scheduled = deploy::ScheduleTarget {
                account_id: account.clone(),
                script_name: script.clone(),
                crons: crons.to_vec(),
            };
            deployments.push(DeployTarget::Schedule(scheduled));
        }

        let durable_objects = match env {
            Some(e) => e.durable_objects.as_ref(),
            None => self.durable_objects.as_ref(),
        };

        if durable_objects.is_none() && deployments.is_empty() {
            StdOut::warn("No deployment routes specified, worker will not be triggered. Please specify your deployment routes or set `workers_dev = true` inside of your configuration file in order to trigger your worker. For more information, see: https://developers.cloudflare.com/workers/cli-wrangler/configuration#keys");
        }

        Ok(deployments)
    }

    pub fn get_account_id(&self, environment_name: Option<&str>) -> Result<String> {
        let environment = self.get_environment(environment_name)?;
        if let Some(environment) = environment {
            if let Some(account_id) = &environment.account_id {
                return Ok(account_id.to_string());
            }
        }
        self.account_id.load().map(String::from)
    }

    pub fn get_target(&self, environment_name: Option<&str>, preview: bool) -> Result<Target> {
        if self.site.is_some() {
            match self.target_type {
                TargetType::Rust => {
                    anyhow::bail!(
                        "{} Workers Sites does not support Rust type projects.",
                        emoji::WARN
                    )
                }
                TargetType::JavaScript => {
                    let error_message = format!(
                        "{} Workers Sites requires using a bundler, and your configuration indicates that you aren't using one. You can fix this by:\n* setting your project type to \"webpack\" to use our automatically configured webpack bundler.\n* setting your project type to \"javascript\", and configuring a build command in the `[build]` section if you wish to use your choice of bundler.",
                        emoji::WARN
                    );
                    if let Some(build) = &self.build {
                        if build.command.is_none() {
                            anyhow::bail!(error_message)
                        }
                    } else {
                        anyhow::bail!(error_message)
                    }
                }
                _ => {}
            }
        }

        /*
        From https://developers.cloudflare.com/workers/cli-wrangler/configuration#keys
        Top level: required to be configured at the top level of your wrangler.toml only; multiple environments on the same project must share this property

        Inherited: Can be configured at the top level and/or environment. If the property is defined only at the top level, the environment will use the property value from the top level. If the property is defined in the environment, the environment value will override the top level value.

        Not inherited: Must be defined for every environment individually.
        */
        let mut target = Target {
            target_type: self.target_type.clone(),       // Top level
            account_id: self.account_id.clone(),         // Inherited
            webpack_config: self.webpack_config.clone(), // Inherited
            build: self.build.clone(),                   // Inherited
            // importantly, the top level name will be modified
            // to include the name of the environment
            name: self.name.clone(), // Inherited
            kv_namespaces: get_namespaces(self.kv_namespaces.clone(), preview)?, // Not inherited
            r2_buckets: get_buckets(self.r2_buckets.clone(), preview)?, // Not inherited
            durable_objects: self.durable_objects.clone(), // Not inherited
            migrations: match (preview, &self.migrations) {
                (false, Some(migrations)) => Some(Migrations::List {
                    script_tag: MigrationTag::Unknown,
                    migrations: migrations.clone(),
                }),
                _ => None,
            }, // Top level
            site: self.site.clone(), // Inherited
            vars: self.vars.clone(), // Not inherited
            text_blobs: self.text_blobs.clone(), // Inherited
            usage_model: self.usage_model, // Top level
            wasm_modules: self.wasm_modules.clone(),
            compatibility_date: self.compatibility_date.clone(),
            compatibility_flags: self.compatibility_flags.clone(),
        };

        let environment = self.get_environment(environment_name)?;

        if let Some(environment) = environment {
            target.name = self.worker_name(environment_name);
            if let Some(account_id) = &environment.account_id {
                target.account_id = Some(account_id.clone()).into();
            }
            if let Some(webpack_config) = &environment.webpack_config {
                target.webpack_config = Some(webpack_config.clone());
            }
            if let Some(build) = &environment.build {
                target.build = Some(build.clone());
            }

            // don't inherit kv namespaces because it is an anti-pattern to use the same namespaces across multiple environments
            target.kv_namespaces = get_namespaces(environment.kv_namespaces.clone(), preview)?;

            // don't inherit r2 buckets because it is an anti-pattern to use the same buckets across multiple environments
            target.r2_buckets = get_buckets(environment.r2_buckets.clone(), preview)?;

            // don't inherit durable object configuration
            target.durable_objects = environment.durable_objects.clone();

            // inherit site configuration
            if let Some(site) = &environment.site {
                target.site = Some(site.clone());
            }

            // don't inherit vars
            target.vars = environment.vars.clone();
        }

        Ok(target)
    }

    pub fn get_environment(&self, environment_name: Option<&str>) -> Result<Option<&Environment>> {
        // check for user-specified environment name
        if let Some(environment_name) = environment_name {
            if let Some(environment_table) = &self.env {
                if let Some(environment) = environment_table.get(environment_name) {
                    Ok(Some(environment))
                } else {
                    anyhow::bail!(
                        "{} Could not find environment with name \"{}\"",
                        emoji::WARN,
                        environment_name
                    )
                }
            } else {
                anyhow::bail!(
                    "{} There are no environments specified in your configuration file",
                    emoji::WARN
                )
            }
        } else {
            Ok(None)
        }
    }

    pub fn warn_about_compatibility_date(&self) {
        if self.compatibility_date.is_some() {
            return;
        }
        let current_date = Utc::now().format("%F");
        let message = &format!(
            r#"
    Your configuration file is missing compatibility_date, so a past date is assumed.
    To get the latest possibly-breaking bug fixes, add this line to your wrangler.toml:

        compatibility_date = "{}"

    For more information, see: https://developers.cloudflare.com/workers/platform/compatibility-dates
        "#,
            current_date
        );
        StdOut::warn(message);
    }

    fn warn_on_account_info(&self) {
        let account_id_env = env::var("CF_ACCOUNT_ID").is_ok();
        let zone_id_env = env::var("CF_ZONE_ID").is_ok();
        let mut top_level_fields: Vec<String> = Vec::new();
        if let Some(kv_namespaces) = &self.kv_namespaces {
            for kv_namespace in kv_namespaces {
                top_level_fields.push(format!(
                    "kv-namespace {} needs a namespace_id",
                    kv_namespace.binding
                ));
            }
        }
        if let Some(route) = &self.route {
            if !route.is_empty() {
                top_level_fields.push("route".to_string());
            }
        }
        if let Some(zone_id) = &self.zone_id {
            if !zone_id.is_empty() && !zone_id_env {
                top_level_fields.push("zone_id".to_string());
            }
        }

        let mut env_fields: HashMap<String, Vec<String>> = HashMap::new();

        if let Some(env) = &self.env {
            for (env_name, env) in env {
                let mut current_env_fields: Vec<String> = Vec::new();
                if env.account_id.is_some() && !account_id_env {
                    current_env_fields.push("account_id".to_string());
                }
                if let Some(kv_namespaces) = &env.kv_namespaces {
                    for kv_namespace in kv_namespaces {
                        current_env_fields.push(format!(
                            "kv-namespace {} needs a namespace_id",
                            kv_namespace.binding
                        ));
                    }
                }
                if let Some(route) = &env.route {
                    if !route.is_empty() {
                        current_env_fields.push("route".to_string());
                    }
                }
                if let Some(zone_id) = &env.zone_id {
                    if !zone_id.is_empty() && !zone_id_env {
                        current_env_fields.push("zone_id".to_string());
                    }
                }
                if !current_env_fields.is_empty() {
                    env_fields.insert(env_name.to_string(), current_env_fields);
                }
            }
        }
        let has_top_level_fields = !top_level_fields.is_empty();
        let has_env_fields = !env_fields.is_empty();
        let mut needs_new_line = false;
        if has_top_level_fields || has_env_fields {
            let toml_msg = styles::highlight("wrangler.toml");
            let zone_id_msg = styles::highlight("zone_id");
            let dash_url = styles::url("https://dash.cloudflare.com");
            let account_id_msg = styles::highlight("account_id");

            StdOut::help(&format!(
                "You can find your {} in the right sidebar of a zone's overview tab at {}",
                zone_id_msg, dash_url
            ));

            whoami::display_account_id_maybe();

            StdOut::help(
                &format!("You will need to update the following fields in the created {} file before continuing:", toml_msg)
            );
            StdOut::help(&format!(
                "You can find your {} in the right sidebar of your account's Workers page, and {} in the right sidebar of a zone's overview tab at {} (if you have only a workers.dev domain, you can skip adding the {} )",
                account_id_msg, zone_id_msg, dash_url, zone_id_msg
            ));

            if has_top_level_fields {
                needs_new_line = true;
                for top_level_field in top_level_fields {
                    println!("- {}", top_level_field);
                }
            }

            if has_env_fields {
                for (env_name, env_fields) in env_fields {
                    if needs_new_line {
                        println!();
                    }
                    println!("[env.{}]", env_name);
                    needs_new_line = true;
                    for env_field in env_fields {
                        println!("  - {}", env_field);
                    }
                }
            }
        }
    }
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct LazyAccountId(OnceCell<String>);

impl Serialize for LazyAccountId {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        self.0.get().serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for LazyAccountId {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        Ok(match Option::<String>::deserialize(deserializer)? {
            None => None,
            Some(x) if x.is_empty() => None,
            Some(x) => Some(x),
        }
        .into())
    }
}

impl From<Option<String>> for LazyAccountId {
    fn from(opt: Option<String>) -> Self {
        let cell = OnceCell::new();
        if let Some(val) = opt {
            cell.set(val).unwrap();
        }
        Self(cell)
    }
}

impl LazyAccountId {
    /// Return the `account_id` in `wrangler.toml`, if present.
    ///
    /// Use this with caution; prefer `maybe_load` instead where possible.
    fn if_present(&self) -> Option<&String> {
        self.0.get()
    }

    /// If `account_id` can be inferred automatically, do so;
    /// otherwise, return `None`.
    ///
    /// Note that *unlike* `load`, this will never prompt the user or warn.
    pub(crate) fn maybe_load(&self) -> Option<String> {
        if let Some(id) = self.0.get() {
            return Some(id.to_owned());
        }

        if let Some(mut accounts) = GlobalUser::new()
            .ok()
            .and_then(|user| fetch_accounts(&user).ok())
        {
            if accounts.len() == 1 {
                return Some(accounts.pop().unwrap().id);
            }
        }

        None
    }

    /// Load the account ID, possibly prompting the user.
    #[cfg_attr(test, allow(unreachable_code))]
    pub(crate) fn load(&self) -> Result<&String> {
        self.0.get_or_try_init(|| {
            #[cfg(test)]
            // don't try to fetch the accounts for this ID, since it's not valid.
            anyhow::bail!("tried to load account id");

            let user = GlobalUser::new()?;
            match fetch_accounts(&user)?.as_slice() {
                [] => {
                    StdOut::user_error("Your authentication token does not match any account ID.");
                    whoami::display_account_id_maybe();
                    anyhow::bail!("field `account_id` is required")
                }
                [single] => Ok(single.id.clone()),
                _multiple => {
                    StdOut::user_error("You have multiple accounts.");
                    whoami::display_account_id_maybe();
                    anyhow::bail!("field `account_id` is required")
                }
            }
        })
    }
}

impl FromStr for Manifest {
    type Err = toml::de::Error;

    fn from_str(serialized_toml: &str) -> Result<Self, Self::Err> {
        toml::from_str(serialized_toml)
    }
}

fn read_config(config_path: &Path) -> Result<Config> {
    let mut config = Config::new();

    let config_str = config_path
        .to_str()
        .expect("project config path should be a string");
    config.merge(File::with_name(config_str))?;

    // Eg.. `CF_ACCOUNT_AUTH_KEY=farts` would set the `account_auth_key` key
    config.merge(config::Environment::with_prefix("CF"))?;

    Ok(config)
}

fn check_for_duplicate_names(manifest: &Manifest) -> Result<()> {
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
        anyhow::bail!(
            "{} Each name in your configuration file must be unique, {}: {}",
            emoji::WARN,
            message,
            duplicate_name_string
        )
    }
    Ok(())
}

fn get_namespaces(
    kv_namespaces: Option<Vec<ConfigKvNamespace>>,
    preview: bool,
) -> Result<Vec<KvNamespace>> {
    if let Some(namespaces) = kv_namespaces {
        namespaces.into_iter().map(|ns| {
            if preview {
                if let Some(preview_id) = &ns.preview_id {
                    if let Some(id) = &ns.id {
                        if preview_id == id {
                            StdOut::warn("Specifying the same KV namespace ID for both preview and production sessions may cause bugs in your production worker! Proceed with caution.");
                        }
                    }
                    Ok(KvNamespace {
                        id: preview_id.to_string(),
                        binding: ns.binding.to_string(),
                    })
                } else {
                    anyhow::bail!("In order to preview a worker with KV namespaces, you must designate a preview_id in your configuration file for each KV namespace you'd like to preview.")
                }
            } else if let Some(id) = &ns.id {
                Ok(KvNamespace {
                    id: id.to_string(),
                    binding: ns.binding,
                })
            } else {
                anyhow::bail!("You must specify the namespace ID in the id field for the namespace with binding \"{}\"", &ns.binding)
            }
        }).collect()
    } else {
        Ok(Vec::new())
    }
}

fn get_buckets(r2_buckets: Option<Vec<ConfigR2Bucket>>, preview: bool) -> Result<Vec<R2Bucket>> {
    if let Some(buckets) = r2_buckets {
        buckets.into_iter().map(|ns| {
            if preview {
                if let Some(preview_bucket_name) = &ns.preview_bucket_name {
                    if let Some(bucket_name) = &ns.bucket_name {
                        if preview_bucket_name == bucket_name {
                            StdOut::warn("Specifying the same r2 bucket_name for both preview and production sessions may cause bugs in your production worker! Proceed with caution.");
                        }
                    }
                    Ok(R2Bucket {
                        bucket_name: preview_bucket_name.to_string(),
                        binding: ns.binding.to_string(),
                    })
                } else {
                    anyhow::bail!("In order to preview a worker with r2 buckets, you must designate a preview_bucket_name in your configuration file for each r2 bucket you'd like to preview.")
                }
            } else if let Some(bucket_name) = &ns.bucket_name {
                Ok(R2Bucket {
                    bucket_name: bucket_name.to_string(),
                    binding: ns.binding,
                })
            } else {
                anyhow::bail!("You must specify the bucket name in the bucket_name field for the bucket with binding \"{}\"", &ns.binding)
            }
        }).collect()
    } else {
        Ok(Vec::new())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn generate() -> Result<()> {
        let toml_path = Path::new(".");

        let toml = Manifest::generate(
            "test".to_string(),
            Some(TargetType::JavaScript),
            toml_path,
            None,
        )?;
        assert_eq!(toml.name, "test".to_string());
        assert_eq!(toml.target_type.to_string(), "javascript".to_string());
        fs::remove_file(toml_path.with_file_name("wrangler.toml"))?;

        let toml = Manifest::generate("test".to_string(), None, toml_path, None)?;
        assert_eq!(toml.target_type.to_string(), "webpack".to_string());
        fs::remove_file(toml_path.with_file_name("wrangler.toml"))?;

        Ok(())
    }

    #[test]
    fn serialize() {
        let manifest = Manifest {
            durable_objects: Some(Default::default()),
            kv_namespaces: Some(vec![ConfigKvNamespace {
                binding: "FOO".to_string(),
                id: Some("123".to_string()),
                preview_id: None,
            }]),
            site: Some(Default::default()),
            vars: Some(
                vec![("FOO".to_string(), "some value".to_string())]
                    .into_iter()
                    .collect(),
            ),
            ..Default::default()
        };
        assert!(toml::to_string(&manifest).is_ok());
    }
}
