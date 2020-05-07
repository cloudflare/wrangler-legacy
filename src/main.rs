#![allow(clippy::redundant_closure)]

extern crate text_io;
extern crate tokio;

use std::env;
use std::path::Path;
use std::str::FromStr;

use clap::{App, AppSettings, Arg, ArgGroup, SubCommand};
use commands::HTTPMethod;
use exitfailure::ExitFailure;

use url::Url;

use wrangler::commands;
use wrangler::commands::kv::key::KVMetaData;
use wrangler::installer;
use wrangler::settings;
use wrangler::settings::global_user::GlobalUser;
use wrangler::settings::toml::TargetType;
use wrangler::terminal::{emoji, interactive, message, styles};

fn main() -> Result<(), ExitFailure> {
    env_logger::init();
    if let Ok(me) = env::current_exe() {
        // If we're actually running as the installer then execute our
        // self-installation, otherwise just continue as usual.
        if me
            .file_stem()
            .and_then(|s| s.to_str())
            .expect("executable should have a filename")
            .starts_with("wrangler-init")
        {
            installer::install();
        }
    }
    Ok(run()?)
}

fn run() -> Result<(), failure::Error> {
    // Define commonly used arguments and arg groups up front for consistency
    // The args below are for KV Subcommands
    let kv_binding_arg = Arg::with_name("binding")
        .help("The binding of the namespace this action applies to")
        .short("b")
        .long("binding")
        .value_name("BINDING NAME")
        .takes_value(true);
    let kv_namespace_id_arg = Arg::with_name("namespace-id")
        .help("The id of the namespace this action applies to")
        .short("n")
        .long("namespace-id")
        .value_name("ID")
        .takes_value(true);
    let kv_namespace_specifier_group = ArgGroup::with_name("namespace-specifier")
        .args(&["binding", "namespace-id"])
        .required(true);

    // This arg is for any action that uses environments (e.g. KV subcommands, publish)
    let environment_arg = Arg::with_name("env")
        .help("Environment to use")
        .short("e")
        .long("env")
        .takes_value(true)
        .value_name("ENVIRONMENT NAME");

    let secret_name_arg = Arg::with_name("name")
        .help("Name of the secret variable")
        .short("n")
        .long("name")
        .required(true)
        .takes_value(true)
        .index(1)
        .value_name("VAR_NAME");

    let matches = App::new(format!("{}{} wrangler", emoji::WORKER, emoji::SPARKLES))
        .version(env!("CARGO_PKG_VERSION"))
        .author("The Wrangler Team <wrangler@cloudflare.com>")
        .setting(AppSettings::ArgRequiredElseHelp)
        .setting(AppSettings::DeriveDisplayOrder)
        .setting(AppSettings::VersionlessSubcommands)
        .subcommand(
            SubCommand::with_name("kv:namespace")
                .about(&*format!(
                    "{} Interact with your Workers KV Namespaces",
                    emoji::FILES
                ))
                .setting(AppSettings::SubcommandRequiredElseHelp)
                .subcommand(
                    SubCommand::with_name("create")
                        .about("Create a new namespace")
                        .arg(environment_arg.clone())
                        .arg(
                            Arg::with_name("binding")
                            .help("The binding for your new namespace")
                            .required(true)
                            .index(1)
                        )
                )
                .subcommand(
                    SubCommand::with_name("delete")
                        .about("Delete namespace")
                        .arg(kv_binding_arg.clone())
                        .arg(kv_namespace_id_arg.clone())
                        .group(kv_namespace_specifier_group.clone())
                        .arg(environment_arg.clone())
                )
                .subcommand(
                    SubCommand::with_name("list")
                        .about("List all namespaces on your Cloudflare account")
                )
        )
        .subcommand(
            SubCommand::with_name("kv:key")
                .about(&*format!(
                    "{} Individually manage Workers KV key-value pairs",
                    emoji::KEY
                ))
                .setting(AppSettings::SubcommandRequiredElseHelp)
                .subcommand(
                    SubCommand::with_name("put")
                        .about("Put a key-value pair into a namespace")
                        .arg(kv_binding_arg.clone())
                        .arg(kv_namespace_id_arg.clone())
                        .group(kv_namespace_specifier_group.clone())
                        .arg(environment_arg.clone())
                        .arg(
                            Arg::with_name("key")
                            .help("Key to write value to")
                            .required(true)
                            .index(1)
                        )
                        .arg(
                            Arg::with_name("value")
                            .help("Value for key")
                            .required(true)
                            .index(2)
                        )
                        .arg(
                            Arg::with_name("expiration-ttl")
                            .help("Number of seconds for which the entries should be visible before they expire. At least 60. Takes precedence over 'expiration' option")
                            .short("t")
                            .long("ttl")
                            .value_name("SECONDS")
                            .takes_value(true)
                        )
                        .arg(
                            Arg::with_name("expiration")
                            .help("Number of seconds since the UNIX epoch, indicating when the key-value pair should expire")
                            .short("x")
                            .long("expiration")
                            .takes_value(true)
                            .value_name("SECONDS")
                        )
                        .arg(
                            Arg::with_name("path")
                            .help("The value passed in is a path to a file; open and upload its contents")
                            .short("p")
                            .long("path")
                            .takes_value(false)
                        )
                )
                .subcommand(
                    SubCommand::with_name("get")
                        .about("Get a key's value from a namespace")
                        .arg(kv_binding_arg.clone())
                        .arg(kv_namespace_id_arg.clone())
                        .group(kv_namespace_specifier_group.clone())
                        .arg(environment_arg.clone())
                        .arg(
                            Arg::with_name("key")
                            .help("Key whose value to get")
                            .required(true)
                            .index(1)
                        )
                )
                .subcommand(
                    SubCommand::with_name("delete")
                        .about("Delete a key and its value from a namespace")
                        .arg(kv_binding_arg.clone())
                        .arg(kv_namespace_id_arg.clone())
                        .group(kv_namespace_specifier_group.clone())
                        .arg(environment_arg.clone())
                        .arg(
                            Arg::with_name("key")
                            .help("Key whose value to delete")
                            .required(true)
                            .index(1)
                        )
                )
                .subcommand(
                    SubCommand::with_name("list")
                        .about("List all keys in a namespace. Produces JSON output")
                        .arg(kv_binding_arg.clone())
                        .arg(kv_namespace_id_arg.clone())
                        .group(kv_namespace_specifier_group.clone())
                        .arg(environment_arg.clone())
                        .arg(
                            Arg::with_name("prefix")
                            .help("The prefix for filtering listed keys")
                            .short("p")
                            .long("prefix")
                            .value_name("STRING")
                            .takes_value(true),
                        )
                )
        )
        .subcommand(
            SubCommand::with_name("kv:bulk")
                .about(&*format!(
                    "{} Interact with multiple Workers KV key-value pairs at once",
                    emoji::BICEP
                ))
                .setting(AppSettings::SubcommandRequiredElseHelp)
                .subcommand(
                    SubCommand::with_name("put")
                        .about("Upload multiple key-value pairs to a namespace")
                        .arg(kv_binding_arg.clone())
                        .arg(kv_namespace_id_arg.clone())
                        .group(kv_namespace_specifier_group.clone())
                        .arg(environment_arg.clone())
                        .arg(
                            Arg::with_name("path")
                            .help("the JSON file of key-value pairs to upload, in form [{\"key\":..., \"value\":...}\"...]")
                            .required(true)
                            .index(1)
                        )
                )
                .subcommand(
                    SubCommand::with_name("delete")
                        .arg(kv_binding_arg.clone())
                        .arg(kv_namespace_id_arg.clone())
                        .group(kv_namespace_specifier_group.clone())
                        .arg(environment_arg.clone())
                        .about("Delete multiple keys and their values from a namespace")
                        .arg(
                            Arg::with_name("path")
                            .help("the JSON file of key-value pairs to upload, in form [\"<example-key>\", ...]")
                            .required(true)
                            .index(1)
                        )
                )
        )
        .subcommand(
            SubCommand::with_name("route")
                .about(&*format!(
                    "{} List or delete worker routes.",
                    emoji::ROUTE
                ))
                .setting(AppSettings::SubcommandRequiredElseHelp)
                .subcommand(
                    SubCommand::with_name("list")
                        .about("List all routes associated with a zone (outputs json)")
                        .arg(environment_arg.clone())
                )
                .subcommand(
                    SubCommand::with_name("delete")
                        .arg(environment_arg.clone())
                        .about("Delete a route by id")
                        .arg(
                            Arg::with_name("route_id")
                            .help("the id associated with the route you want to delete (find using `wrangler route list`)")
                            .required(true)
                            .index(1)
                        )
                )
        )
        .subcommand(
            SubCommand::with_name("secret")
                .about(&*format!(
                    "{} Generate a secret that can be referenced in the worker script",
                    emoji::SECRET
                ))
                .setting(AppSettings::SubcommandRequiredElseHelp)
                .subcommand(
                    SubCommand::with_name("put")
                        .about("Create or update a secret variable for a script")
                        .arg(secret_name_arg.clone())
                        .arg(environment_arg.clone())
                )
                .subcommand(
                    SubCommand::with_name("delete")
                        .about("Delete a secret variable from a script")
                        .arg(secret_name_arg.clone())
                        .arg(environment_arg.clone())
                )
                .subcommand(
                    SubCommand::with_name("list")
                        .about("List all secrets for a script")
                        .arg(environment_arg.clone())
                )
        )
        .subcommand(
            SubCommand::with_name("generate")
                .about(&*format!(
                    "{} Generate a new worker project",
                    emoji::DANCERS
                ))
                .arg(
                    Arg::with_name("name")
                        .help("the name of your worker! defaults to 'worker'")
                        .index(1),
                )
                .arg(
                    Arg::with_name("template")
                        .help("a link to a github template! defaults to https://github.com/cloudflare/worker-template")
                        .index(2),
                )
                .arg(
                    Arg::with_name("type")
                        .short("t")
                        .long("type")
                        .takes_value(true)
                        .help("the type of project you want generated"),
                )
                .arg(
                    Arg::with_name("site")
                        .short("s")
                        .long("site")
                        .takes_value(false)
                        .help("initializes a Workers Sites project. Overrides `type` and `template`"),
                ),
        )
        .subcommand(
            SubCommand::with_name("init")
                .about(&*format!(
                    "{} Create a wrangler.toml for an existing project",
                    emoji::INBOX
                ))
                .arg(
                    Arg::with_name("name")
                        .help("the name of your worker! defaults to 'worker'")
                        .index(1),
                )
                .arg(
                    Arg::with_name("type")
                        .short("t")
                        .long("type")
                        .takes_value(true)
                        .help("the type of project you want generated"),
                )
                .arg(
                    Arg::with_name("site")
                        .short("s")
                        .long("site")
                        .takes_value(false)
                        .help("initializes a Workers Sites project. Overrides `type` and `template`"),
                ),
        )
        .subcommand(
            SubCommand::with_name("build")
                .about(&*format!(
                    "{} Build your worker",
                    emoji::CRAB
                ))
                .arg(
                    Arg::with_name("env")
                        .help("environment to build")
                        .short("e")
                        .long("env")
                        .takes_value(true)
                ),
        )
        .subcommand(
            SubCommand::with_name("preview")
                .about(&*format!(
                    "{} Preview your code temporarily on cloudflareworkers.com",
                    emoji::MICROSCOPE
                ))
                .arg(
                    Arg::with_name("headless")
                        .help("Don't open the browser on preview")
                        .long("headless")
                        .takes_value(false)
                )
                .arg(
                    Arg::with_name("method")
                        .help("Type of request to preview your worker with (get, post)")
                        .index(1),
                )
                .arg(
                    Arg::with_name("body")
                        .help("Body string to post to your preview worker request")
                        .index(2),
                )
                .arg(
                    Arg::with_name("url")
                        .help("URL to open in the worker preview")
                        .short("u")
                        .long("url")
                        .takes_value(true)
                )
                .arg(
                    Arg::with_name("env")
                        .help("Environment to preview")
                        .short("e")
                        .long("env")
                        .takes_value(true)
                )
                .arg(
                    Arg::with_name("watch")
                        .help("Watch your project for changes and update the preview automagically")
                        .long("watch")
                        .takes_value(false),
                )
                .arg(
                    Arg::with_name("verbose")
                        .long("verbose")
                        .takes_value(false)
                        .help("Toggle verbose output"),
                ),
        )
        .subcommand(
            SubCommand::with_name("dev")
                .about(&*format!(
                    "{} Start a local server for developing your worker",
                    emoji::EAR
                ))
                .arg(
                    Arg::with_name("env")
                        .help("environment to build")
                        .short("e")
                        .long("env")
                        .takes_value(true)
                )
                .arg(
                    Arg::with_name("port")
                        .help("port to listen on. defaults to 8787")
                        .short("p")
                        .long("port")
                        .takes_value(true)
                )
                .arg(
                    Arg::with_name("host")
                        .help("domain to test behind your worker. defaults to example.com")
                        .short("h")
                        .long("host")
                        .takes_value(true)
                )
                .arg(
                    Arg::with_name("ip")
                        .help("ip to listen on. defaults to localhost")
                        .short("i")
                        .long("ip")
                        .takes_value(true)
                )
                .arg(
                    Arg::with_name("verbose")
                        .long("verbose")
                        .takes_value(false)
                        .help("toggle verbose output")
                ),
        )
        .subcommand(
            SubCommand::with_name("publish")
                .about(&*format!(
                    "{} Publish your worker to the orange cloud",
                    emoji::UP
                ))
                .arg(
                    Arg::with_name("env")
                        .help("environments to publish to")
                        .short("e")
                        .long("env")
                        .takes_value(true)
                )
                .arg(
                    Arg::with_name("verbose")
                        .long("verbose")
                        .takes_value(false)
                        .help("toggle verbose output")
                )
                .arg(
                    Arg::with_name("release")
                        .long("release")
                        .takes_value(false)
                        .help("[deprecated] alias of wrangler publish")
                ),
        )
        .subcommand(
            SubCommand::with_name("config")
                .about(&*format!(
                    "{} Set up wrangler with your Cloudflare account",
                    emoji::SLEUTH
                ))
                .arg(
                    Arg::with_name("api-key")
                        .help("use an email and global API key for authentication. This is not recommended; use API tokens (the default) if possible")
                        .long("api-key")
                        .takes_value(false),
                )
                .arg(
                    Arg::with_name("no-verify")
                        .help("do not verify provided credentials before writing out Wrangler config file")
                        .long("no-verify")
                        .takes_value(false),
                ),
        )
        .subcommand(
            SubCommand::with_name("subdomain")
                .about(&*format!(
                    "{} Configure your workers.dev subdomain",
                    emoji::WORKER
                ))
                .arg(
                    Arg::with_name("name")
                        .help("the subdomain on workers.dev you'd like to reserve")
                        .index(1),
                ),
        )
        .subcommand(SubCommand::with_name("whoami").about(&*format!(
            "{} Retrieve your user info and test your auth config",
            emoji::SLEUTH
        )))
        .subcommand(
            SubCommand::with_name("tail")
                .about(&*format!("{} Aggregate logs from production worker", emoji::TAIL))
                .arg(
                    Arg::with_name("env")
                        .help("environment to tail logs from")
                        .short("e")
                        .long("env")
                        .takes_value(true)
                )
                .arg(
                    Arg::with_name("tunnel_port")
                        .help("port to accept tail log requests")
                        .short("p")
                        .long("port")
                        .takes_value(true)
                )
                .arg(
                    Arg::with_name("metrics_port")
                        .help("provides endpoint for cloudflared metrics. used to retrieve tunnel url")
                        .long("metrics")
                        .takes_value(true)
                )
                .arg(
                    Arg::with_name("verbose")
                        .long("verbose")
                        .takes_value(false)
                        .help("Toggle verbose output"),
                )
        )
        .get_matches();

    let config_path = Path::new("./wrangler.toml");

    let not_recommended_msg = styles::warning("(Not Recommended)");
    let recommended_cmd_msg = styles::highlight("`wrangler config --api-key`");
    let api_token_url = styles::url("https://dash.cloudflare.com/profile/api-tokens");
    let token_support_url = styles::url(
        "https://support.cloudflare.com/hc/en-us/articles/200167836-Managing-API-Tokens-and-Keys",
    );

    if let Some(matches) = matches.subcommand_matches("config") {
        // If api-key flag isn't present, use the default auth option (API token)
        let default = !matches.is_present("api-key");

        let user: GlobalUser = if default {
            // API Tokens are the default
            message::billboard(&format!("To find your API Token, go to {}\nand create it using the \"Edit Cloudflare Workers\" template.\n\nIf you are trying to use your Global API Key instead of an API Token\n{}, run {}.", api_token_url, not_recommended_msg, recommended_cmd_msg));
            let api_token: String = interactive::get_user_input("Enter API Token: ");
            GlobalUser::TokenAuth { api_token }
        } else {
            message::billboard(&format!("We don't recommend using your Global API Key!\nPlease consider using an API Token instead.\n\n{}", token_support_url));
            let email: String = interactive::get_user_input("Enter Email: ");
            let api_key: String = interactive::get_user_input("Enter Global API Key: ");

            GlobalUser::GlobalKeyAuth { email, api_key }
        };

        let verify = !matches.is_present("no-verify");

        commands::global_config(&user, verify)?;
    } else if let Some(matches) = matches.subcommand_matches("generate") {
        let name = matches.value_of("name").unwrap_or("worker");
        let site = matches.is_present("site");
        let template = matches.value_of("template");
        let mut target_type = None;

        let template = if site {
            if template.is_some() {
                failure::bail!("You cannot pass a template and the --site flag to wrangler generate. If you'd like to use the default site boilerplate, run wrangler generate --site. If you'd like to use another site boilerplate, omit --site when running wrangler generate.")
            }
            "https://github.com/cloudflare/worker-sites-template"
        } else {
            if let Some(type_value) = matches.value_of("type") {
                target_type = Some(TargetType::from_str(&type_value.to_lowercase())?);
            }

            let default_template = "https://github.com/cloudflare/worker-template";
            template.unwrap_or(match target_type {
                Some(ref pt) => match pt {
                    TargetType::Rust => "https://github.com/cloudflare/rustwasm-worker-template",
                    _ => default_template,
                },
                _ => default_template,
            })
        };

        log::info!(
            "Generate command called with template {}, and name {}",
            template,
            name
        );

        commands::generate(name, template, target_type, site)?;
    } else if let Some(matches) = matches.subcommand_matches("init") {
        let name = matches.value_of("name");
        let site = matches.is_present("site");
        let target_type = if site {
            // Workers Sites projects are always webpack for now
            Some(TargetType::Webpack)
        } else {
            match matches.value_of("type") {
                Some(s) => Some(settings::toml::TargetType::from_str(&s.to_lowercase())?),
                None => None,
            }
        };

        commands::init(name, target_type, site)?;
    } else if let Some(matches) = matches.subcommand_matches("build") {
        log::info!("Getting project settings");
        let manifest = settings::toml::Manifest::new(config_path)?;
        let env = matches.value_of("env");
        let target = &manifest.get_target(env)?;
        commands::build(&target)?;
    } else if let Some(matches) = matches.subcommand_matches("preview") {
        log::info!("Getting project settings");
        let manifest = settings::toml::Manifest::new(config_path)?;
        let env = matches.value_of("env");
        let target = manifest.get_target(env)?;

        // the preview command can be called with or without a Global User having been config'd
        // so we convert this Result into an Option
        let user = settings::global_user::GlobalUser::new().ok();

        let method = HTTPMethod::from_str(matches.value_of("method").unwrap_or("get"))?;

        let url = Url::parse(matches.value_of("url").unwrap_or("https://example.com"))?;

        // Validate the URL scheme
        failure::ensure!(
            match url.scheme() {
                "http" => true,
                "https" => true,
                _ => false,
            },
            "Invalid URL scheme (use either \"https\" or \"http\")"
        );

        let body = match matches.value_of("body") {
            Some(s) => Some(s.to_string()),
            None => None,
        };

        let watch = matches.is_present("watch");
        let verbose = matches.is_present("verbose");
        let headless = matches.is_present("headless");

        commands::preview(target, user, method, url, body, watch, verbose, headless)?;
    } else if let Some(matches) = matches.subcommand_matches("dev") {
        log::info!("Starting dev server");
        let port = matches.value_of("port");
        let host = matches.value_of("host");
        let ip = matches.value_of("ip");
        let manifest = settings::toml::Manifest::new(config_path)?;
        let env = matches.value_of("env");
        let target = manifest.get_target(env)?;
        let user = settings::global_user::GlobalUser::new().ok();
        let verbose = matches.is_present("verbose");
        commands::dev::dev(target, user, host, port, ip, verbose)?;
    } else if matches.subcommand_matches("whoami").is_some() {
        log::info!("Getting User settings");
        let user = settings::global_user::GlobalUser::new()?;

        commands::whoami(&user)?;
    } else if let Some(matches) = matches.subcommand_matches("publish") {
        log::info!("Getting User settings");
        let user = settings::global_user::GlobalUser::new()?;

        let release = matches.is_present("release");
        if release {
            let publish_release_msg = styles::highlight("`wrangler publish --release`");
            let publish_msg = styles::highlight("`wrangler publish`");
            let environments_url = styles::url("https://developers.cloudflare.com/workers/tooling/wrangler/configuration/environments");
            message::warn(&format!(
                "{} is deprecated and behaves exactly the same as {}.",
                publish_release_msg, publish_msg
            ));
            message::warn(&format!("See {} for more information.", environments_url));
        }

        log::info!("Getting project settings");
        let manifest = settings::toml::Manifest::new(config_path)?;
        let env = matches.value_of("env");
        let mut target = manifest.get_target(env)?;
        let deploy_config = manifest.deploy_config(env)?;

        let verbose = matches.is_present("verbose");

        commands::publish(&user, &mut target, deploy_config, verbose)?;
    } else if let Some(matches) = matches.subcommand_matches("subdomain") {
        log::info!("Getting project settings");
        let manifest = settings::toml::Manifest::new(config_path)?;
        let env = matches.value_of("env");
        let target = manifest.get_target(env)?;

        log::info!("Getting User settings");
        let user = settings::global_user::GlobalUser::new()?;

        let name = matches.value_of("name");

        if let Some(name) = name {
            commands::subdomain::set_subdomain(&name, &user, &target)?;
        } else {
            commands::subdomain::get_subdomain(&user, &target)?;
        }
    } else if let Some(route_matches) = matches.subcommand_matches("route") {
        let user = settings::global_user::GlobalUser::new()?;
        let manifest = settings::toml::Manifest::new(config_path)?;
        let env = matches.value_of("env");

        let env_zone_id = if let Some(environment) = manifest.get_environment(env)? {
            environment.zone_id.as_ref()
        } else {
            None
        };

        let zone_id: Result<String, failure::Error> = if let Some(zone_id) = env_zone_id {
            Ok(zone_id.to_string())
        } else if let Some(zone_id) = manifest.zone_id {
            Ok(zone_id)
        } else {
            failure::bail!(
                "You must specify a zone_id in `wrangler.toml` to use `wrangler route` commands."
            )
        };

        match route_matches.subcommand() {
            ("list", Some(_)) => {
                commands::route::list(zone_id?, &user)?;
            }
            ("delete", Some(delete_matches)) => {
                let route_id = delete_matches.value_of("route_id").unwrap();
                commands::route::delete(zone_id?, &user, route_id)?;
            }
            ("", None) => message::warn("route expects a subcommand"),
            _ => unreachable!(),
        }
    } else if let Some(secrets_matches) = matches.subcommand_matches("secret") {
        log::info!("Getting project settings");
        let manifest = settings::toml::Manifest::new(config_path)?;
        log::info!("Getting User settings");
        let user = settings::global_user::GlobalUser::new()?;

        match secrets_matches.subcommand() {
            ("put", Some(create_matches)) => {
                let name = create_matches.value_of("name");
                let env = create_matches.value_of("env");
                let target = manifest.get_target(env)?;
                if let Some(name) = name {
                    commands::secret::create_secret(&name, &user, &target)?;
                }
            }
            ("delete", Some(delete_matches)) => {
                let name = delete_matches.value_of("name");
                let env = delete_matches.value_of("env");
                let target = manifest.get_target(env)?;
                if let Some(name) = name {
                    commands::secret::delete_secret(&name, &user, &target)?;
                }
            }
            ("list", Some(list_matches)) => {
                let env = list_matches.value_of("env");
                let target = manifest.get_target(env)?;
                commands::secret::list_secrets(&user, &target)?;
            }
            ("", None) => message::warn("secret expects a subcommand"),
            _ => unreachable!(),
        }
    } else if let Some(kv_matches) = matches.subcommand_matches("kv:namespace") {
        let manifest = settings::toml::Manifest::new(config_path)?;
        let user = settings::global_user::GlobalUser::new()?;

        match kv_matches.subcommand() {
            ("create", Some(create_matches)) => {
                let env = create_matches.value_of("env");
                let target = manifest.get_target(env)?;
                let binding = create_matches.value_of("binding").unwrap();
                commands::kv::namespace::create(&target, env, &user, binding)?;
            }
            ("delete", Some(delete_matches)) => {
                let env = delete_matches.value_of("env");
                let target = manifest.get_target(env)?;
                let namespace_id = match delete_matches.value_of("binding") {
                    Some(namespace_binding) => {
                        commands::kv::get_namespace_id(&target, namespace_binding)?
                    }
                    None => delete_matches
                        .value_of("namespace-id")
                        .unwrap() // clap configs ensure that if "binding" isn't present, "namespace-id" must be.
                        .to_string(),
                };
                commands::kv::namespace::delete(&target, &user, &namespace_id)?;
            }
            ("list", Some(list_matches)) => {
                let env = list_matches.value_of("env");
                let target = manifest.get_target(env)?;
                commands::kv::namespace::list(&target, &user)?;
            }
            ("", None) => message::warn("kv:namespace expects a subcommand"),
            _ => unreachable!(),
        }
    } else if let Some(kv_matches) = matches.subcommand_matches("kv:key") {
        let manifest = settings::toml::Manifest::new(config_path)?;
        let user = settings::global_user::GlobalUser::new()?;

        // Get environment and bindings
        let (subcommand, subcommand_matches) = kv_matches.subcommand();
        let (target, namespace_id) = match subcommand_matches {
            Some(subcommand_matches) => {
                let env = subcommand_matches.value_of("env");
                let target = manifest.get_target(env)?;
                let namespace_id = match subcommand_matches.value_of("binding") {
                    Some(namespace_binding) => {
                        commands::kv::get_namespace_id(&target, namespace_binding)?
                    }
                    None => subcommand_matches
                        .value_of("namespace-id")
                        .unwrap() // clap configs ensure that if "binding" isn't present,"namespace-id" must be.
                        .to_string(),
                };
                (target, namespace_id)
            }
            None => unreachable!(), // this is unreachable because all kv:key commands have required arguments.
        };

        match (subcommand, subcommand_matches) {
            ("get", Some(get_key_matches)) => {
                let key = get_key_matches.value_of("key").unwrap();
                commands::kv::key::get(&target, &user, &namespace_id, key)?
            }
            ("put", Some(put_key_matches)) => {
                let key = put_key_matches.value_of("key").unwrap().to_string();

                // If is_file is true, overwrite value to be the contents of the given
                // filename in the 'value' arg.
                let value = put_key_matches.value_of("value").unwrap().to_string();
                let is_file = put_key_matches.is_present("path");
                let expiration = put_key_matches
                    .value_of("expiration")
                    .map(|e| e.to_string());
                let expiration_ttl = put_key_matches
                    .value_of("expiration-ttl")
                    .map(|t| t.to_string());
                let kv_metadata = KVMetaData {
                    namespace_id,
                    key,
                    value,
                    is_file,
                    expiration,
                    expiration_ttl,
                };
                commands::kv::key::put(&target, &user, kv_metadata)?
            }
            ("delete", Some(delete_key_matches)) => {
                let key = delete_key_matches.value_of("key").unwrap();
                commands::kv::key::delete(&target, &user, &namespace_id, key)?
            }
            ("list", Some(list_key_matches)) => {
                let prefix = list_key_matches.value_of("prefix");
                commands::kv::key::list(&target, &user, &namespace_id, prefix)?
            }
            ("", None) => message::warn("kv:key expects a subcommand"),
            _ => unreachable!(),
        }
    } else if let Some(kv_matches) = matches.subcommand_matches("kv:bulk") {
        let manifest = settings::toml::Manifest::new(config_path)?;
        let user = settings::global_user::GlobalUser::new()?;

        // Get environment and bindings
        let (subcommand, subcommand_matches) = kv_matches.subcommand();
        let (target, namespace_id) = match subcommand_matches {
            Some(subcommand_matches) => {
                let env = subcommand_matches.value_of("env");
                let target = manifest.get_target(env)?;
                let namespace_id = match subcommand_matches.value_of("binding") {
                    Some(namespace_binding) => {
                        commands::kv::get_namespace_id(&target, namespace_binding)?
                    }
                    None => subcommand_matches
                        .value_of("namespace-id")
                        .unwrap() // clap configs ensure that if "binding" isn't present,"namespace-id" must be.
                        .to_string(),
                };
                (target, namespace_id)
            }
            None => unreachable!(), // this is unreachable because all kv:key commands have required arguments.
        };

        match (subcommand, subcommand_matches) {
            ("put", Some(put_bulk_matches)) => {
                let path = put_bulk_matches.value_of("path").unwrap();
                commands::kv::bulk::put(&target, &user, &namespace_id, Path::new(path))?
            }
            ("delete", Some(delete_bulk_matches)) => {
                let path = delete_bulk_matches.value_of("path").unwrap();
                commands::kv::bulk::delete(&target, &user, &namespace_id, Path::new(path))?
            }
            ("", None) => message::warn("kv:bulk expects a subcommand"),
            _ => unreachable!(),
        }
    } else if let Some(matches) = matches.subcommand_matches("tail") {
        let manifest = settings::toml::Manifest::new(config_path)?;
        let env = matches.value_of("env");
        let target = manifest.get_target(env)?;
        let user = settings::global_user::GlobalUser::new()?;

        let tunnel_port: Option<u16> = matches
            .value_of("tunnel_port")
            .map(|p| p.parse().expect("--port expects a number"));
        let metrics_port: Option<u16> = matches
            .value_of("metrics_port")
            .map(|p| p.parse().expect("--metrics expects a number"));

        let verbose = matches.is_present("verbose");

        commands::tail::start(&target, &user, tunnel_port, metrics_port, verbose)?;
    }
    Ok(())
}
