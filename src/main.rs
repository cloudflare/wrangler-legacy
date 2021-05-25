#![cfg_attr(feature = "strict", deny(warnings))]

extern crate text_io;
extern crate tokio;

use std::env;
use std::net::{IpAddr, Ipv4Addr};
use std::path::PathBuf;

use anyhow::{anyhow, ensure, Result};
use clap::{AppSettings, ArgGroup};
use structopt::StructOpt;
use url::Url;

use wrangler::build_target;
use wrangler::commands;
use wrangler::commands::dev::Protocol;
use wrangler::commands::kv::key::{parse_metadata, KVMetaData};
use wrangler::installer;
use wrangler::preview::{HttpMethod, PreviewOpt};
use wrangler::reporter;
use wrangler::settings;
use wrangler::settings::global_user::GlobalUser;
use wrangler::settings::toml::migrations::{
    DurableObjectsMigration, Migration, MigrationConfig, Migrations, RenameClass, TransferClass,
};
use wrangler::settings::toml::{Manifest, TargetType};
use wrangler::terminal::message::{Message, Output, StdOut};
use wrangler::terminal::{interactive, styles};
use wrangler::version::background_check_for_updates;

fn main() -> Result<()> {
    reporter::init();
    env_logger::init();

    let latest_version_receiver = background_check_for_updates();
    if let Ok(me) = env::current_exe() {
        // If we're actually running as the installer then execute our
        // self-installation, otherwise just continue as usual.
        if me
            .file_stem()
            .and_then(|s| s.to_str())
            .expect("executable should have a filename")
            .starts_with("wrangler-init")
        {
            installer::install()?;
        }
    }
    run()?;
    if let Ok(latest_version) = latest_version_receiver.try_recv() {
        let latest_version = styles::highlight(latest_version.to_string());
        let new_version_available = format!(
            "A new version of Wrangler ({}) is available!",
            latest_version
        );
        let update_message = "You can learn more about updating here:".to_string();
        let update_docs_url = styles::url(
            "https://developers.cloudflare.com/workers/cli-wrangler/install-update#update",
        );

        StdOut::billboard(&format!(
            "{}\n{}\n{}",
            new_version_available, update_message, update_docs_url
        ));
    }
    Ok(())
}

#[derive(Debug, StructOpt)]
#[structopt(
    name = "wrangler",
    author = "The Wrangler Team <wrangler@cloudflare.com>",
    setting = AppSettings::ArgRequiredElseHelp,
    setting = AppSettings::DeriveDisplayOrder,
    setting = AppSettings::VersionlessSubcommands,
)]
pub struct Cli {
    /// Toggle verbose output (when applicable)
    #[structopt(name = "verbose", long = "verbose", global = true)]
    pub verbose: bool,

    /// Path to configuration file.
    #[structopt(
        name = "config",
        short = "c",
        long = "config",
        default_value = "./wrangler.toml",
        global = true
    )]
    pub config: PathBuf,

    /// Environment to perform a command on.
    #[structopt(name = "env", short = "e", long = "env", global = true)]
    pub environment: Option<String>,

    #[structopt(subcommand)]
    pub command: Command,
}

#[derive(Debug, StructOpt)]
pub enum Command {
    /// Interact with your Workers KV Namespaces
    #[structopt(name = "kv:namespace", setting = AppSettings::SubcommandRequiredElseHelp)]
    KvNamespace(KvNamespace),

    /// Individually manage Workers KV key-value pairs
    #[structopt(name = "kv:key", setting = AppSettings::SubcommandRequiredElseHelp)]
    KvKey {
        #[structopt(flatten)]
        namespace: Namespace,

        #[structopt(subcommand)]
        subcommand: KvKey,
    },

    /// Interact with multiple Workers KV key-value pairs at once
    #[structopt(name = "kv:bulk", setting = AppSettings::SubcommandRequiredElseHelp)]
    KvBulk {
        #[structopt(flatten)]
        namespace: Namespace,

        #[structopt(subcommand)]
        subcommand: KvBulk,
    },

    /// List or delete worker routes.
    #[structopt(name = "route", setting = AppSettings::SubcommandRequiredElseHelp)]
    Route(Route),

    /// Generate a secret that can be referenced in the worker script
    #[structopt(name = "secret", setting = AppSettings::SubcommandRequiredElseHelp)]
    Secret(Secret),

    /// Generate a new worker project
    Generate {
        /// The name of your worker! defaults to 'worker'
        #[structopt(index = 1, default_value = "worker")]
        name: String,

        /// A link to a GitHub template! Defaults to https://github.com/cloudflare/worker-template
        #[structopt(index = 2)]
        template: Option<String>,

        /// The type of project you want generated
        #[structopt(name = "type", long, short = "t")]
        target_type: Option<TargetType>,

        /// Initializes a Workers Sites project.
        #[structopt(long, short = "s")]
        site: bool,
    },

    /// Create a wrangler.toml for an existing project
    Init {
        /// The name of your worker! Defaults to 'worker'
        #[structopt(index = 1)]
        name: Option<String>,

        /// The type of project you want generated
        #[structopt(name = "type", long, short = "t")]
        target_type: Option<TargetType>,

        /// Initializes a Workers Sites project. Overrides `type` and `template`
        #[structopt(long, short = "s")]
        site: bool,
    },

    /// Build your worker
    Build,

    /// Preview your code temporarily on cloudflareworkers.com
    Preview {
        /// Type of request to preview your worker with (get, post)
        #[structopt(index = 1, default_value = "get")]
        method: HttpMethod,

        /// URL to open in the worker preview
        #[structopt(short = "s", long, default_value = "https://example.com")]
        url: Url,

        /// Body string to post to your preview worker request
        #[structopt(index = 2)]
        body: Option<String>,

        /// Watch your project for changes and update the preview automagically
        #[structopt(long)]
        watch: bool,

        /// Don't open the browser on preview
        #[structopt(long)]
        headless: bool,
    },

    /// Start a local server for developing your worker
    Dev {
        /// Host to forward requests to, defaults to the zone of project or to
        /// tutorial.cloudflareworkers.com if unauthenticated.
        #[structopt(long, short = "h")]
        host: Option<String>,

        /// IP to listen on. Defaults to 127.0.0.1
        #[structopt(long, short = "i")]
        ip: Option<IpAddr>,

        /// Port to listen on. Defaults to 8787
        #[structopt(long, short = "p")]
        port: Option<u16>,

        /// Sets the protocol on which the wrangler dev listens, by default this is http
        /// but can be set to https
        #[structopt(name = "local-protocol")]
        local_protocol: Option<Protocol>,

        /// Sets the protocol on which requests are sent to the host, by default this is https
        /// but can be set to http
        #[structopt(name = "upstream-protocol")]
        upstream_protocol: Option<Protocol>,
    },

    /// Publish your worker to the orange cloud
    #[structopt(name = "publish")]
    Publish {
        /// [deprecated] alias of wrangler publish
        #[structopt(long, hidden = true)]
        release: bool,

        output: Option<String>,

        #[structopt(flatten)]
        migration: AdhocMigration,
    },

    /// Authenticate Wrangler with a Cloudflare API Token or Global API Key
    #[structopt(name = "config")]
    Config {
        /// Use an email and global API key for authentication.
        /// This is not recommended; use API tokens (the default) if possible
        #[structopt(name = "api-key", long)]
        api_key: bool,
        /// Do not verify provided credentials before writing out Wrangler config file
        #[structopt(name = "no-verify", long)]
        no_verify: bool,
    },

    /// Configure your workers.dev subdomain
    #[structopt(name = "subdomain")]
    Subdomain {
        /// The subdomain on workers.dev you'd like to reserve
        #[structopt(name = "name", index = 1)]
        name: Option<String>,
    },

    /// Retrieve your user info and test your auth config
    #[structopt(name = "whoami")]
    Whoami,

    /// Aggregate logs from production worker
    #[structopt(name = "tail")]
    Tail {
        /// Specify an output format
        #[structopt(long, short = "f", default_value = "json", possible_values = &["json", "pretty"])]
        format: String,

        /// Port to accept tail log requests
        #[structopt(long = "port", short = "p")]
        tunnel_port: Option<u16>,

        /// Provides endpoint for cloudflared metrics. Used to retrieve tunnel url
        #[structopt(long = "metrics")]
        metrics_port: Option<u16>,
    },

    /// Authenticate Wrangler with your Cloudflare username and password
    #[structopt(name = "login")]
    Login,

    /// Report an error caught by wrangler to Cloudflare
    #[structopt(name = "report")]
    Report {
        /// Specifies a log to report (e.g. --log=1619728882567.log)
        #[structopt(name = "log", long)]
        log: Option<PathBuf>,
    },
}

#[derive(Debug, StructOpt)]
pub struct AdhocMigration {
    /// Allow durable objects to be created from a class in your script
    #[structopt(name = "new-class", long, number_of_values = 1)]
    new_class: Vec<String>,

    /// Delete all durable objects associated with a class in your script
    #[structopt(name = "delete-class", long, number_of_values = 1)]
    delete_class: Vec<String>,

    /// Rename a durable object class
    #[structopt(name = "rename-class", long, number_of_values = 2)]
    rename_class: Vec<String>,

    /// Transfer all durable objects associated with a class in another script to a class in
    /// this script
    #[structopt(name = "transfer-class", long, number_of_values = 3)]
    transfer_class: Vec<String>,
}

impl AdhocMigration {
    pub fn into_migration_config(self) -> Option<MigrationConfig> {
        let migration = DurableObjectsMigration {
            new_classes: self.new_class,
            deleted_classes: self.delete_class,
            renamed_classes: self
                .rename_class
                .chunks_exact(2)
                .map(|chunk| {
                    let (from, to) = if let [from, to] = chunk {
                        (from.clone(), to.clone())
                    } else {
                        unreachable!("Chunks exact returned a slice with a length not equal to 2")
                    };

                    RenameClass { from, to }
                })
                .collect(),
            transferred_classes: self
                .transfer_class
                .chunks_exact(3)
                .map(|chunk| {
                    let (from_script, from, to) = if let [from_script, from, to] = chunk {
                        (from_script.clone(), from.clone(), to.clone())
                    } else {
                        unreachable!("Chunks exact returned a slice with a length not equal to 3")
                    };

                    TransferClass {
                        from,
                        from_script,
                        to,
                    }
                })
                .collect(),
        };
        let is_migration_empty = migration.new_classes.is_empty()
            && migration.deleted_classes.is_empty()
            && migration.renamed_classes.is_empty()
            && migration.transferred_classes.is_empty();

        if !is_migration_empty {
            Some(MigrationConfig {
                tag: None,
                migration: Migration {
                    durable_objects: migration,
                },
            })
        } else {
            None
        }
    }
}

#[derive(Debug, StructOpt)]
#[structopt(rename_all = "lower")]
pub enum KvNamespace {
    /// Create a new namespace
    Create {
        #[structopt(name = "binding", long, short = "b")]
        binding: String,
        /// Applies the command to the preview namespace when combined with 'binding'
        #[structopt(name = "preview", long)]
        preview: bool,
    },
    /// Delete namespace
    Delete {
        #[structopt(flatten)]
        namespace: Namespace,
    },
    /// List all namespaces on your Cloudflare account
    List,
}

#[derive(Debug, StructOpt)]
#[structopt(group = ArgGroup::with_name("namespace-specifier").required(true))]
pub struct Namespace {
    /// The binding of the namespace this action applies to
    #[structopt(long, short = "b", group = "namespace-specifier", global = true)]
    pub binding: Option<String>,

    /// Applies the command to the preview namespace when combined with --binding
    #[structopt(long, requires = "binding", global = true)]
    pub preview: bool,

    /// The ID of the namespace this action applies to
    #[structopt(
        name = "namespace-id",
        long,
        short = "n",
        group = "namespace-specifier",
        global = true
    )]
    pub namespace_id: Option<String>,
}

#[derive(Debug, StructOpt)]
#[structopt(rename_all = "lower")]
pub enum KvKey {
    /// Put a key-value pair into a namespace
    Put {
        /// Key to write the value to
        #[structopt(name = "key", index = 1)]
        key: String,

        /// Value for key
        #[structopt(name = "value", index = 2)]
        value: String,

        /// Number of seconds for which the entries should be visible before they expire.
        /// At least 60. Takes precedence over 'expiration' option.
        #[structopt(name = "expiration-ttl", short = "t", long = "ttl")]
        expiration_ttl: Option<u64>,

        /// Number of seconds since the UNIX epoch, indicating when the key-value pair should expire.
        #[structopt(name = "expiration", long, short = "x")]
        expiration: Option<u64>,

        /// Arbitrary JSON to associate with a key-value pair. Must be no more than 1024 bytes.
        #[structopt(name = "metadata", long, short = "m")]
        metadata: Option<String>,

        /// The value passed in is a path to a file; open and upload its contents
        #[structopt(name = "path", long, short = "p")]
        path: bool,
    },
    /// Get a key's value from a namespace
    Get {
        /// Key whose value to get
        #[structopt(name = "key", index = 1)]
        key: String,
    },
    /// Delete a key and its value from a namespace
    Delete {
        /// Key whose value to get
        #[structopt(name = "key", index = 1)]
        key: String,
    },
    /// List all keys in a namespace. Produces JSON output
    List {
        /// The prefix for filtering listed keys
        #[structopt(name = "prefix", long, short = "p")]
        prefix: Option<String>,
    },
}

#[derive(Debug, StructOpt)]
#[structopt(rename_all = "lower")]
pub enum KvBulk {
    /// Upload multiple key-value pairs to a namespace
    Put {
        /// The JSON file of key-value pairs to upload, in form [{\"key\":..., \"value\":...}\"...]
        #[structopt(index = 1)]
        path: PathBuf,
    },
    /// Delete multiple keys and their values from a namespace
    Delete {
        /// The JSON file of key-value pairs to upload, in form [\"<example-key>\", ...]
        #[structopt(index = 1)]
        path: PathBuf,
    },
}

#[derive(Debug, StructOpt)]
#[structopt(rename_all = "lower")]
pub enum Route {
    /// List all routes associated with a zone (outputs json)
    List,
    /// Delete a route by ID
    Delete {
        /// The ID associated with the route you want to delete (find using `wrangler route list`)
        #[structopt(index = 1)]
        route_id: String,
    },
}

#[derive(Debug, StructOpt)]
#[structopt(rename_all = "lower")]
pub enum Secret {
    /// Create or update a secret variable for a script
    Put {
        #[structopt(long, short = "n", index = 1)]
        name: String,
    },
    /// Delete a secret variable from a script
    Delete {
        #[structopt(long, short = "n", index = 1)]
        name: String,
    },
    /// List all secrets for a script
    List,
}

fn run() -> Result<()> {
    let cli = Cli::from_args();

    match cli.command {
        Command::Config { api_key, no_verify } => {
            let user: GlobalUser = if !api_key {
                // API Tokens are the default
                StdOut::billboard(&format!(concat!(
                    "To find your API Token, go to {}\n",
                    "and create it using the \"Edit Cloudflare Workers\" template.\n",
                    "\n",
                    "Consider using {} which only requires your Cloudflare username and password.\n",
                    "\n",
                    "If you are trying to use your Global API Key instead of an API Token\n",
                    "{}, run {}."),
                    styles::url("https://dash.cloudflare.com/profile/api-tokens"),
                    styles::highlight("`wrangler login`"),
                    styles::warning("(Not Recommended)"),
                    styles::highlight("`wrangler config --api-key`")
                ));
                let api_token: String = interactive::get_user_input("Enter API Token: ");
                GlobalUser::TokenAuth { api_token }
            } else {
                StdOut::billboard(&format!(concat!(
                    "We don't recommend using your Global API Key!\n",
                    "Please consider using an API Token instead.\n",
                    "\n",
                    "{}"),
                    styles::url(
                        "https://support.cloudflare.com/hc/en-us/articles/200167836-Managing-API-Tokens-and-Keys",
                    )
                ));
                let email: String = interactive::get_user_input("Enter Email: ");
                let api_key: String = interactive::get_user_input("Enter Global API Key: ");

                GlobalUser::GlobalKeyAuth { email, api_key }
            };

            commands::global_config(&user, !no_verify)?;
        }
        Command::Generate {
            name,
            site,
            template,
            target_type,
        } => {
            const DEFAULT_TEMPLATE: &str = "https://github.com/cloudflare/worker-template";
            const RUST_TEMPLATE: &str = "https://github.com/cloudflare/rustwasm-worker-template";
            const SITES_TEMPLATE: &str = "https://github.com/cloudflare/worker-sites-template";

            let template = if site {
                SITES_TEMPLATE
            } else if let Some(template) = template.as_deref() {
                template
            } else if let Some(TargetType::Rust) = target_type {
                RUST_TEMPLATE
            } else {
                DEFAULT_TEMPLATE
            };

            log::info!(
                "Generate command called with template {}, and name {}",
                template,
                name
            );

            commands::generate(&name, template, target_type, site)?;
        }
        Command::Init {
            name,
            site,
            target_type,
        } => {
            let target_type = if site {
                // Workers Sites projects are always webpack for now
                Some(TargetType::Webpack)
            } else {
                target_type
            };

            commands::init(name.as_deref(), target_type, site)?;
        }
        Command::Build => {
            log::info!("Getting project settings");
            let manifest = Manifest::new(&cli.config)?;
            let target = manifest.get_target(cli.environment.as_deref(), false)?;
            build_target(&target).map(|msg| StdOut::success(&msg))?;
        }
        Command::Preview {
            method,
            url,
            body,
            watch,
            headless,
        } => {
            log::info!("Getting project settings");
            let manifest = Manifest::new(&cli.config)?;
            let target = manifest.get_target(cli.environment.as_deref(), true)?;

            // the preview command can be called with or without a Global User having been config'd
            // so we convert this Result into an Option
            let user = settings::global_user::GlobalUser::new().ok();

            // Validate the URL scheme
            ensure!(
                matches!(url.scheme(), "http" | "https"),
                "Invalid URL scheme (use either \"https\" or \"http\")"
            );

            let options = PreviewOpt {
                method,
                url,
                body,
                livereload: watch,
                headless,
            };

            commands::preview(target, user, options, cli.verbose)?;
        }
        Command::Dev {
            host,
            mut ip,
            mut port,
            mut local_protocol,
            mut upstream_protocol,
        } => {
            log::info!("Starting dev server");
            let manifest = Manifest::new(&cli.config)?;

            // Check if arg not given but present in wrangler.toml
            if let Some(d) = &manifest.dev {
                ip = ip.or(d.ip);
                port = port.or(d.port);
                local_protocol = local_protocol.or(d.local_protocol);
                upstream_protocol = upstream_protocol.or(d.upstream_protocol);
            }

            let ip = ip.unwrap_or_else(|| Ipv4Addr::new(127, 0, 0, 1).into());
            let port = port.unwrap_or(8787);
            let local_protocol = local_protocol.unwrap_or(Protocol::Http);
            let upstream_protocol = upstream_protocol.unwrap_or(Protocol::Https);

            let deployments = manifest.get_deployments(cli.environment.as_deref())?;
            let target = manifest.get_target(cli.environment.as_deref(), true)?;
            let user = settings::global_user::GlobalUser::new().ok();

            let server_config =
                commands::dev::ServerConfig::new(host, ip, port, upstream_protocol)?;

            commands::dev::dev(
                target,
                deployments,
                user,
                server_config,
                local_protocol,
                upstream_protocol,
                cli.verbose,
            )?;
        }
        Command::Whoami => {
            log::info!("Getting User settings");
            let user = settings::global_user::GlobalUser::new()?;

            commands::whoami(&user)?;
        }
        Command::Publish {
            release,
            output,
            migration,
        } => {
            log::info!("Getting User settings");
            let user = settings::global_user::GlobalUser::new()?;

            if release {
                StdOut::warn(&format!(concat!(
                    "{} is deprecated and behaves exactly the same as {}.\n",
                    "See {} for more information."),
                    styles::highlight("`wrangler publish --release`"),
                    styles::highlight("`wrangler publish`"),
                    styles::url("https://developers.cloudflare.com/workers/tooling/wrangler/configuration/environments"),
                ));
            }

            log::info!("Getting project settings");
            let manifest = Manifest::new(&cli.config)?;
            let mut target = manifest.get_target(cli.environment.as_deref(), false)?;

            if let Some(migration) = migration.into_migration_config() {
                target.migrations = Some(Migrations {
                    migrations: vec![migration],
                });
            }

            let output = if output.as_deref() == Some("json") {
                Output::Json
            } else {
                Output::PlainText
            };
            let deploy_config = manifest.get_deployments(cli.environment.as_deref())?;
            commands::publish(&user, &mut target, deploy_config, output)?;
        }
        Command::Subdomain { name } => {
            log::info!("Getting project settings");
            let manifest = Manifest::new(&cli.config)?;
            let target = manifest.get_target(cli.environment.as_deref(), false)?;

            log::info!("Getting User settings");
            let user = settings::global_user::GlobalUser::new()?;

            if let Some(name) = name {
                commands::subdomain::set_subdomain(&name, &user, &target)?;
            } else {
                commands::subdomain::get_subdomain(&user, &target)?;
            }
        }
        Command::Route(route) => {
            let user = settings::global_user::GlobalUser::new()?;
            let manifest = Manifest::new(&cli.config)?;
            let zone_id = manifest
                .get_environment(cli.environment.as_deref())?
                .and_then(|e| e.zone_id.as_ref())
                .or_else(|| manifest.zone_id.as_ref());

            let zone_id = zone_id.ok_or_else(|| anyhow::anyhow!(
                "You must specify a zone_id in your configuration file to use `wrangler route` commands."
            ))?;

            match route {
                Route::List => {
                    commands::route::list(zone_id, &user)?;
                }
                Route::Delete { route_id } => {
                    commands::route::delete(zone_id, &user, &route_id)?;
                }
            }
        }
        Command::Secret(secret) => {
            log::info!("Getting User settings");
            let user = settings::global_user::GlobalUser::new()?;

            log::info!("Getting project settings");
            let manifest = Manifest::new(&cli.config)?;
            let target = manifest.get_target(cli.environment.as_deref(), false)?;
            match secret {
                Secret::Put { name } => {
                    commands::secret::create_secret(&name, &user, &target)?;
                }
                Secret::Delete { name } => {
                    commands::secret::delete_secret(&name, &user, &target)?;
                }
                Secret::List => {
                    commands::secret::list_secrets(&user, &target)?;
                }
            }
        }
        Command::KvNamespace(namespace) => {
            let user = settings::global_user::GlobalUser::new()?;
            let manifest = Manifest::new(&cli.config)?;
            let env = cli.environment.as_deref();

            match namespace {
                KvNamespace::Create { binding, preview } => {
                    commands::kv::namespace::create(&manifest, preview, env, &user, &binding)?;
                }
                KvNamespace::Delete { namespace } => {
                    let target = manifest.get_target(env, namespace.preview)?;
                    let id = if let Some(binding) = namespace.binding {
                        commands::kv::get_namespace_id(&target, &binding)?
                    } else {
                        namespace
                            .namespace_id
                            .expect("Namespace ID is required if binding isn't supplied")
                    };
                    commands::kv::namespace::delete(&target, &user, &id)?;
                }
                KvNamespace::List => {
                    let target = manifest.get_target(env, false)?;
                    commands::kv::namespace::list(&target, &user)?;
                }
            }
        }
        Command::KvKey {
            namespace,
            subcommand,
        } => {
            let user = settings::global_user::GlobalUser::new()?;
            let manifest = Manifest::new(&cli.config)?;
            let env = cli.environment.as_deref();

            let target = manifest.get_target(env, namespace.preview)?;
            let namespace_id = if let Some(binding) = namespace.binding {
                commands::kv::get_namespace_id(&target, &binding)?
            } else {
                namespace
                    .namespace_id
                    .expect("Namespace ID is required if binding isn't supplied")
            };

            match subcommand {
                KvKey::Get { key } => commands::kv::key::get(&target, &user, &namespace_id, &key)?,
                KvKey::Put {
                    key,
                    value,
                    path: is_file,
                    expiration_ttl,
                    expiration,
                    metadata,
                } => {
                    let expiration = expiration.as_ref().map(ToString::to_string);
                    let expiration_ttl = expiration_ttl.as_ref().map(ToString::to_string);
                    let metadata = parse_metadata(metadata.as_deref())
                        .map_err(|e| anyhow!("--metadata is not valid JSON: {}", e.to_string()))?;

                    commands::kv::key::put(
                        &target,
                        &user,
                        KVMetaData {
                            namespace_id,
                            key,
                            value,
                            is_file,
                            expiration,
                            expiration_ttl,
                            metadata,
                        },
                    )?;
                }
                KvKey::Delete { key } => {
                    commands::kv::key::delete(&target, &user, &namespace_id, &key)?
                }
                KvKey::List { prefix } => {
                    commands::kv::key::list(&target, &user, &namespace_id, prefix.as_deref())?
                }
            }
        }
        Command::KvBulk {
            namespace,
            subcommand,
        } => {
            // Get environment and bindings
            let manifest = Manifest::new(&cli.config)?;
            let user = settings::global_user::GlobalUser::new()?;
            let env = cli.environment.as_deref();

            let target = manifest.get_target(env, namespace.preview)?;
            let namespace_id = if let Some(binding) = namespace.binding {
                commands::kv::get_namespace_id(&target, &binding)?
            } else {
                namespace
                    .namespace_id
                    .expect("Namespace ID is required if binding isn't supplied")
            };

            match subcommand {
                KvBulk::Put { path } => {
                    commands::kv::bulk::put(&target, &user, &namespace_id, &path)?
                }
                KvBulk::Delete { path } => {
                    commands::kv::bulk::delete(&target, &user, &namespace_id, &path)?
                }
            }
        }
        Command::Tail {
            format,
            tunnel_port,
            metrics_port,
        } => {
            let manifest = Manifest::new(&cli.config)?;
            let target = manifest.get_target(cli.environment.as_deref(), false)?;
            let user = settings::global_user::GlobalUser::new()?;

            commands::tail::start(
                &target,
                &user,
                format,
                tunnel_port,
                metrics_port,
                cli.verbose,
            )?;
        }
        Command::Login => {
            commands::login::run()?;
        }
        Command::Report { log } => {
            commands::report::run(log.as_deref()).map(|_| {
                eprintln!("Report submission sucessful. Thank you!");
            })?;
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn rename_class(tag: &str) -> RenameClass {
        RenameClass {
            from: format!("renameFrom{}", tag),
            to: format!("renameTo{}", tag),
        }
    }

    fn transfer_class(tag: &str) -> TransferClass {
        TransferClass {
            from: format!("transferFromClass{}", tag),
            from_script: format!("transferFromScript{}", tag),
            to: format!("transferToClass{}", tag),
        }
    }

    #[test]
    fn adhoc_migration_parsing() {
        let command = Cli::from_iter(&[
            "wrangler",
            "publish",
            "--new-class",
            "newA",
            "--new-class",
            "newB",
            "--delete-class",
            "deleteA",
            "--delete-class",
            "deleteB",
            "--rename-class",
            "renameFromA",
            "renameToA",
            "--rename-class",
            "renameFromB",
            "renameToB",
            "--transfer-class",
            "transferFromScriptA",
            "transferFromClassA",
            "transferToClassA",
            "--transfer-class",
            "transferFromScriptB",
            "transferFromClassB",
            "transferToClassB",
        ])
        .command;

        if let Command::Publish { migration, .. } = command {
            assert_eq!(
                migration.into_migration_config(),
                Some(MigrationConfig {
                    tag: None,
                    migration: Migration {
                        durable_objects: DurableObjectsMigration {
                            new_classes: vec![String::from("newA"), String::from("newB")],
                            deleted_classes: vec![String::from("deleteA"), String::from("deleteB")],
                            renamed_classes: vec![rename_class("A"), rename_class("B")],
                            transferred_classes: vec![transfer_class("A"), transfer_class("B")],
                        }
                    }
                })
            );
        } else {
            assert!(false, "Unkown command {:?}", command)
        }
    }
}
