#![allow(clippy::redundant_closure)]

#[macro_use]
extern crate text_io;

use std::env;
use std::path::Path;
use std::str::FromStr;

use commands::HTTPMethod;
use exitfailure::ExitFailure;

use wrangler::commands;
use wrangler::commands::kv::key::KVMetaData;
use wrangler::installer;
use wrangler::settings::{self, global_user::GlobalUser, toml::TargetType};
use wrangler::terminal::message;

mod cli;

use cli::cli_prelude::{App, ArgMatches};

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
    Ok(run(cli::app())?)
}

// Run the cli application.
fn run(app: App) -> Result<(), failure::Error> {
    let matches = app.get_matches();

    let config_path = Path::new("./wrangler.toml");

    // Match on the given sub-command and delegate
    // to the appropriate handler.
    match matches.subcommand() {
        ("config", Some(matches)) => {
            config(&matches)?;
        }
        ("generate", Some(matches)) => {
            generate(&matches)?;
        }
        ("init", Some(matches)) => {
            init(&matches)?;
        }
        ("build", Some(matches)) => {
            build(&matches, &config_path)?;
        }
        ("preview", Some(matches)) => {
            preview(&matches, config_path)?;
        }
        ("whoami", Some(_)) => {
            whoami()?;
        }
        ("publish", Some(matches)) => {
            publish(&matches, config_path)?;
        }
        ("subdomain", Some(matches)) => {
            subdomain(&matches, config_path)?;
        }
        ("kv:namespace", Some(matches)) => {
            kv_namespace(&matches, config_path)?;
        }
        ("kv:key", Some(matches)) => {
            kv_key(&matches, config_path)?;
        }
        ("kv:bulk", Some(matches)) => {
            kv_bulk(&matches, config_path)?;
        }
        // I don't know if this is actually unreachable. However, there should be an
        // error returned in this case: SubCommandMatchDoesNotExist ({_}, {_}).
        (_, _) => unreachable!(),
    }
    Ok(())
}

fn config(matches: &ArgMatches) -> Result<(), failure::Error> {
    // If api-key flag isn't present, use the default auth option (API token)
    let default = !matches.is_present("api-key");

    let user: GlobalUser = if default {
        // API Tokens are the default
        message::big_info("To find your API token, go to https://dash.cloudflare.com/profile/api-tokens\n\tand create it using the \"Edit Cloudflare Workers\" template");
        message::big_info("If you are trying to use your Global API Key instead of an API Token\n\t(Not Recommended), run \"wrangler config --api-key\".\n");
        println!("Enter API token: ");
        let mut api_token: String = read!("{}\n");
        api_token.truncate(api_token.trim_end().len());
        GlobalUser::TokenAuth { api_token }
    } else {
        message::big_info("We don't recommend using your Global API Key! Please consider using an\n\tAPI Token instead.\n\thttps://support.cloudflare.com/hc/en-us/articles/200167836-Managing-API-Tokens-and-Keys\n");
        println!("Enter email: ");
        let mut email: String = read!("{}\n");
        email.truncate(email.trim_end().len());

        println!("Enter global API key: ");
        let mut api_key: String = read!("{}\n");
        api_key.truncate(api_key.trim_end().len());

        GlobalUser::GlobalKeyAuth { email, api_key }
    };

    let verify = !matches.is_present("no-verify");

    Ok(commands::global_config(&user, verify)?)
}

fn generate(matches: &ArgMatches) -> Result<(), failure::Error> {
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

    Ok(commands::generate(name, template, target_type, site)?)
}

fn init(matches: &ArgMatches) -> Result<(), failure::Error> {
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

    Ok(commands::init(name, target_type, site)?)
}

fn build(matches: &ArgMatches, config_path: &Path) -> Result<(), failure::Error> {
    log::info!("Getting project settings");
    let manifest = settings::toml::Manifest::new(config_path)?;
    let env = matches.value_of("env");
    let target = manifest.get_target(env)?;
    Ok(commands::build(&target)?)
}

fn preview(matches: &ArgMatches, config_path: &Path) -> Result<(), failure::Error> {
    log::info!("Getting project settings");
    let manifest = settings::toml::Manifest::new(config_path)?;
    let env = matches.value_of("env");
    let target = manifest.get_target(env)?;

    // the preview command can be called with or without a Global User having been config'd
    // so we convert this Result into an Option
    let user = settings::global_user::GlobalUser::new().ok();

    let method = HTTPMethod::from_str(matches.value_of("method").unwrap_or("get"))?;

    let body = match matches.value_of("body") {
        Some(s) => Some(s.to_string()),
        None => None,
    };

    let watch = matches.is_present("watch");
    let verbose = matches.is_present("verbose");
    let headless = matches.is_present("headless");

    Ok(commands::preview(
        target, user, method, body, watch, verbose, headless,
    )?)
}

fn whoami() -> Result<(), failure::Error> {
    log::info!("Getting User settings");
    let user = settings::global_user::GlobalUser::new()?;

    Ok(commands::whoami(&user)?)
}

fn publish(matches: &ArgMatches, config_path: &Path) -> Result<(), failure::Error> {
    log::info!("Getting User settings");
    let user = settings::global_user::GlobalUser::new()?;

    let release = matches.is_present("release");
    if release {
        message::warn("wrangler publish --release is deprecated and behaves exactly the same as wrangler publish.");
        message::warn("See https://developers.cloudflare.com/workers/tooling/wrangler/configuration/environments for more information.");
    }

    log::info!("Getting project settings");
    let manifest = settings::toml::Manifest::new(config_path)?;
    let env = matches.value_of("env");
    let mut target = manifest.get_target(env)?;

    let verbose = matches.is_present("verbose");

    Ok(commands::publish(&user, &mut target, verbose)?)
}

fn subdomain(matches: &ArgMatches, config_path: &Path) -> Result<(), failure::Error> {
    log::info!("Getting project settings");
    let manifest = settings::toml::Manifest::new(config_path)?;
    let env = matches.value_of("env");
    let target = manifest.get_target(env)?;

    log::info!("Getting User settings");
    let user = settings::global_user::GlobalUser::new()?;

    let name = matches.value_of("name");

    if let Some(name) = name {
        Ok(commands::subdomain::set_subdomain(&name, &user, &target)?)
    } else {
        Ok(commands::subdomain::get_subdomain(&user, &target)?)
    }
}

fn kv_namespace(kv_matches: &ArgMatches, config_path: &Path) -> Result<(), failure::Error> {
    let manifest = settings::toml::Manifest::new(config_path)?;
    let user = settings::global_user::GlobalUser::new()?;

    match kv_matches.subcommand() {
        ("create", Some(create_matches)) => {
            let env = create_matches.value_of("env");
            let target = manifest.get_target(env)?;
            let binding = create_matches.value_of("binding").unwrap();
            Ok(commands::kv::namespace::create(
                &target, env, &user, binding,
            )?)
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
            Ok(commands::kv::namespace::delete(
                &target,
                &user,
                &namespace_id,
            )?)
        }
        ("list", Some(list_matches)) => {
            let env = list_matches.value_of("env");
            let target = manifest.get_target(env)?;
            Ok(commands::kv::namespace::list(&target, &user)?)
        }
        ("", None) => Ok(message::warn("kv:namespace expects a subcommand")),
        _ => unreachable!(),
    }
}

fn kv_key(kv_matches: &ArgMatches, config_path: &Path) -> Result<(), failure::Error> {
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
            Ok(commands::kv::key::get(&target, &user, &namespace_id, key)?)
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
            Ok(commands::kv::key::put(&target, &user, kv_metadata)?)
        }
        ("delete", Some(delete_key_matches)) => {
            let key = delete_key_matches.value_of("key").unwrap();
            Ok(commands::kv::key::delete(
                &target,
                &user,
                &namespace_id,
                key,
            )?)
        }
        ("list", Some(list_key_matches)) => {
            let prefix = list_key_matches.value_of("prefix");
            Ok(commands::kv::key::list(
                &target,
                &user,
                &namespace_id,
                prefix,
            )?)
        }
        ("", None) => Ok(message::warn("kv:key expects a subcommand")),
        _ => unreachable!(),
    }
}

fn kv_bulk(kv_matches: &ArgMatches, config_path: &Path) -> Result<(), failure::Error> {
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
            Ok(commands::kv::bulk::put(
                &target,
                &user,
                &namespace_id,
                Path::new(path),
            )?)
        }
        ("delete", Some(delete_bulk_matches)) => {
            let path = delete_bulk_matches.value_of("path").unwrap();
            Ok(commands::kv::bulk::delete(
                &target,
                &user,
                &namespace_id,
                Path::new(path),
            )?)
        }
        ("", None) => Ok(message::warn("kv:bulk expects a subcommand")),
        _ => unreachable!(),
    }
}
