#![allow(clippy::redundant_closure)]

#[macro_use]
extern crate text_io;

use std::env;
use std::path::Path;
use std::str::FromStr;

use clap::{App, AppSettings, Arg, SubCommand};
use commands::HTTPMethod;

use log::info;

mod commands;
mod http;
mod install;
mod installer;
mod settings;
mod terminal;
mod util;

use crate::settings::project::ProjectType;
use exitfailure::ExitFailure;
use terminal::emoji;
use terminal::message;

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
    let matches = App::new(format!("{}{} wrangler", emoji::WORKER, emoji::SPARKLES))
        .version(env!("CARGO_PKG_VERSION"))
        .author("ashley g williams <ashley666ashley@gmail.com>")
        .setting(AppSettings::ArgRequiredElseHelp)
        .setting(AppSettings::DeriveDisplayOrder)
        .subcommand(
            SubCommand::with_name("kv:namespace")
                .about(&*format!(
                    "{} Interact with your Workers KV Namespaces",
                    emoji::KV
                ))
                .subcommand(
                    SubCommand::with_name("create")
                        .about("Create a new namespace")
                        .arg(
                            Arg::with_name("title")
                            .help("The name for your new namespace")
                            .required(true)
                        )
                )
                .subcommand(
                    SubCommand::with_name("delete")
                        .about("Delete namespace")
                        .arg(
                            Arg::with_name("namespace-id")
                            .help("The ID of the namespace this action applies to")
                            .required(true)
                        )
                )
                .subcommand(
                    SubCommand::with_name("rename")
                        .about("Rename a namespace")
                        .arg(
                            Arg::with_name("namespace-id")
                            .help("The ID of the namespace this action applies to")
                            .required(true)
                        )
                        .arg(
                            Arg::with_name("title")
                            .help("New title for the namespace")
                            .required(true)
                        )
                )
                .subcommand(
                    SubCommand::with_name("list")
                        .about("List all namespaces on your Cloudflare account")
                )
        )
            .subcommand(SubCommand::with_name("kv:key")
                .about(&*format!(
                    "{} Interact with your Workers KV Key-Value Pairs",
                    emoji::KV
                ))
                .subcommand(
                    SubCommand::with_name("put")
                        .about("Put a key-value pair into a namespace")
                        .arg(
                            Arg::with_name("namespace-id")
                            .help("The ID of the namespace this action applies to")
                            .required(true)
                            // .short("n")
                            // .long("namespace-id")
                            // .value_name("<ID>")
                            // .takes_value(true)
                        )
                        .arg(
                            Arg::with_name("key")
                            .help("Key to write value to")
                            .required(true)
                        )
                        .arg(
                            Arg::with_name("value")
                            .help("Value for key")
                            .required(true)
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
                            .short("e")
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
                        .arg(
                            Arg::with_name("namespace-id")
                            .help("The ID of the namespace this action applies to")
                            .required(true)
                            // .short("n")
                            // .long("namespace-id")
                            // .value_name("<ID>")
                            // .takes_value(true)
                        )
                        .arg(
                            Arg::with_name("key")
                            .help("Key whose value to get")
                            .required(true)
                        )
                )
                .subcommand(
                    SubCommand::with_name("delete")
                        .about("Delete a key and its value from a namespace")
                        .arg(
                            Arg::with_name("namespace-id")
                            .help("The ID of the namespace this action applies to")
                            .required(true)
                            // .short("n")
                            // .long("namespace-id")
                            // .value_name("<ID>")
                            // .takes_value(true)
                        )
                        .arg(
                            Arg::with_name("key")
                            .help("Key whose value to delete")
                            .required(true)
                        )
                )
                .subcommand(
                    SubCommand::with_name("list")
                        .about("List all keys in a namespace. Produces JSON output")
                        .arg(
                            Arg::with_name("namespace-id")
                            .help("The ID of the namespace this action applies to")
                            .required(true)
                            // .short("n")
                            // .long("namespace-id")
                            // .value_name("<ID>")
                            // .takes_value(true)
                        )
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
                    "{} Interact with your Workers KV Key-Value Pairs",
                    emoji::KV
                ))
                .subcommand(
                    SubCommand::with_name("put")
                        .about("Upload multiple key-value pairs to a namespace")
                        .arg(
                            Arg::with_name("namespace-id")
                                .help("The ID of the namespace this action applies to")
                                .required(true)
                                // .short("n")
                                // .long("namespace-id")
                                // .value_name("<ID>")
                                // .takes_value(true)
                        )
                        .arg(
                            Arg::with_name("path")
                            .help("the JSON file of key-value pairs to upload, in form [{\"key\":..., \"value\":...}\"...]")
                            .required(true)
                            .index(2),
                        )
                )
                .subcommand(
                    SubCommand::with_name("delete")
                        .about("Delete multiple keys and their values from a namespace")
                        .arg(
                            Arg::with_name("namespace-id")
                            .help("The ID of the namespace this action applies to")
                            .required(true)
                            // .short("n")
                            // .long("namespace-id")
                            // .value_name("<ID>")
                            // .takes_value(true)
                        )
                        .arg(
                            Arg::with_name("path")
                            .help("the JSON file of key-value pairs to upload, in form [\"<example-key>\", ...]")
                            .required(true)
                        )
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
                        .help("a link to a github template! defaults to cloudflare/worker-template")
                        .index(2),
                )
                .arg(
                    Arg::with_name("type")
                        .short("t")
                        .long("type")
                        .takes_value(true)
                        .help("the type of project you want generated"),
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
                ),
        )
        .subcommand(
            SubCommand::with_name("build")
                .about(&*format!(
                    "{} Build your worker",
                    emoji::CRAB
                ))
        )
        .subcommand(
            SubCommand::with_name("preview")
                .about(&*format!(
                    "{} Preview your code temporarily on cloudflareworkers.com",
                    emoji::MICROSCOPE
                ))
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
                    Arg::with_name("watch")
                        .help("watch your project for changes and update the preview automagically")
                        .long("watch")
                        .takes_value(false),
                )
        )
        .subcommand(
            SubCommand::with_name("publish").about(&*format!(
                "{} Publish your worker to the orange cloud",
                emoji::UP
            ))
            .arg(
                Arg::with_name("release")
                    .long("release")
                    .takes_value(false)
                    .help("should this be published to a workers.dev subdomain or a domain name you have registered"),
             ),
        )
        .subcommand(
            SubCommand::with_name("config")
                .about(&*format!(
                    "{} Setup wrangler with your Cloudflare account",
                    emoji::SLEUTH
                )),
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
                        .index(1)
                        .required(true),
                ),
        )
        .subcommand(SubCommand::with_name("whoami").about(&*format!(
            "{} Retrieve your user info and test your auth config",
            emoji::SLEUTH
        )))
        .get_matches();

    if let Some(_matches) = matches.subcommand_matches("config") {
        println!("Enter email: ");
        let mut email: String = read!("{}\n");
        email.truncate(email.trim_end().len());
        println!("Enter api key: ");
        let mut api_key: String = read!("{}\n");
        api_key.truncate(api_key.trim_end().len());

        commands::global_config(email, api_key)?;
    } else if let Some(matches) = matches.subcommand_matches("generate") {
        let name = matches.value_of("name").unwrap_or("worker");
        let project_type = match matches.value_of("type") {
            Some(s) => Some(ProjectType::from_str(&s.to_lowercase())?),
            None => None,
        };

        let default_template = "https://github.com/cloudflare/worker-template";
        let template = matches.value_of("template").unwrap_or(match project_type {
            Some(ref pt) => match pt {
                ProjectType::Rust => "https://github.com/cloudflare/rustwasm-worker-template",
                _ => default_template,
            },
            _ => default_template,
        });

        info!(
            "Generate command called with template {}, and name {}",
            template, name
        );
        commands::generate(name, template, project_type)?;
    } else if let Some(matches) = matches.subcommand_matches("init") {
        let name = matches.value_of("name");
        let project_type = match matches.value_of("type") {
            Some(s) => Some(settings::project::ProjectType::from_str(&s.to_lowercase())?),
            None => None,
        };
        commands::init(name, project_type)?;
    } else if matches.subcommand_matches("build").is_some() {
        info!("Getting project settings");
        let project = settings::project::Project::new()?;

        commands::build(&project)?;
    } else if let Some(matches) = matches.subcommand_matches("preview") {
        info!("Getting project settings");
        let project = settings::project::Project::new()?;

        // the preview command can be called with or without a Global User having been config'd
        // so we convert this Result into an Option
        let user = settings::global_user::GlobalUser::new().ok();

        let method = HTTPMethod::from_str(matches.value_of("method").unwrap_or("get"))?;

        let body = match matches.value_of("body") {
            Some(s) => Some(s.to_string()),
            None => None,
        };

        let watch = matches.is_present("watch");

        commands::preview(project, user, method, body, watch)?;
    } else if matches.subcommand_matches("whoami").is_some() {
        info!("Getting User settings");
        let user = settings::global_user::GlobalUser::new()?;

        commands::whoami(&user);
    } else if let Some(matches) = matches.subcommand_matches("publish") {
        info!("Getting project settings");
        let project = settings::project::Project::new()?;

        info!("Getting User settings");
        let user = settings::global_user::GlobalUser::new()?;

        info!("{}", matches.occurrences_of("release"));
        let release = match matches.occurrences_of("release") {
            1 => true,
            _ => false,
        };

        commands::publish(&user, &project, release)?;
    } else if let Some(matches) = matches.subcommand_matches("subdomain") {
        info!("Getting project settings");
        let project = settings::project::Project::new()?;

        info!("Getting User settings");
        let user = settings::global_user::GlobalUser::new()?;

        let name = matches
            .value_of("name")
            .expect("The subdomain name you are requesting must be provided.");

        commands::subdomain(name, &user, &project)?;
    } else if let Some(kv_matches) = matches.subcommand_matches("kv:namespace") {
        match kv_matches.subcommand() {
            ("create", Some(create_matches)) => {
                let title = create_matches.value_of("title").unwrap();
                commands::kv::create_namespace(title)?;
            }
            ("delete", Some(delete_matches)) => {
                let id = delete_matches.value_of("namespace-id").unwrap();
                commands::kv::delete_namespace(id)?;
            }
            ("rename", Some(rename_matches)) => {
                let id = rename_matches.value_of("namespace-id").unwrap();
                let title = rename_matches.value_of("title").unwrap();
                commands::kv::rename_namespace(id, title)?;
            }
            ("list", Some(_list_matches)) => {
                commands::kv::list_namespaces()?;
            }
            ("", None) => message::warn("kv:namespace expects a subcommand"),
            _ => unreachable!(),
        }
    } else if let Some(kv_matches) = matches.subcommand_matches("kv:key") {
        match kv_matches.subcommand() {
            ("get", Some(read_key_matches)) => {
                let project = settings::project::Project::new()?;
                let user = settings::global_user::GlobalUser::new()?;
                let id = read_key_matches.value_of("namespace-id").unwrap();
                let key = read_key_matches.value_of("key").unwrap();
                commands::kv::read_key(&project, &user, id, key)?;
            }
            ("put", Some(write_key_matches)) => {
                let project = settings::project::Project::new()?;
                let user = settings::global_user::GlobalUser::new()?;
                let id = write_key_matches.value_of("namespace-id").unwrap();
                let key = write_key_matches.value_of("key").unwrap();
                let value = write_key_matches.value_of("value").unwrap();
                let is_file = match write_key_matches.occurrences_of("path") {
                    1 => true,
                    _ => false,
                };
                let expiration = write_key_matches.value_of("expiration");
                let ttl = write_key_matches.value_of("expiration-ttl");
                commands::kv::write_key(&project, &user, id, key, value, is_file, expiration, ttl)?;
            }
            ("delete", Some(delete_matches)) => {
                let id = delete_matches.value_of("namespace-id").unwrap();
                let key = delete_matches.value_of("key").unwrap();
                commands::kv::delete_key(id, key)?;
            }
            ("list", Some(list_keys_matches)) => {
                let id = list_keys_matches.value_of("namespace-id").unwrap();
                let prefix = list_keys_matches.value_of("prefix");
                commands::kv::list_keys(id, prefix)?;
            }
            ("", None) => message::warn("kv:key expects a subcommand"),
            _ => unreachable!(),
        }
    } else if let Some(kv_matches) = matches.subcommand_matches("kv:bulk") {
        match kv_matches.subcommand() {
            ("put", Some(write_bulk_matches)) => {
                let id = write_bulk_matches.value_of("namespace-id").unwrap();
                let path = write_bulk_matches.value_of("path").unwrap();
                commands::kv::write_json(id, Path::new(path))?;
            }
            ("delete", Some(delete_matches)) => {
                let id = delete_matches.value_of("namespace-id").unwrap();
                let path = delete_matches.value_of("path").unwrap();
                commands::kv::delete_json(id, Path::new(path))?;
            }
            ("", None) => message::warn("kv:bulk expects a subcommand"),
            _ => unreachable!(),
        }
    }
    Ok(())
}
