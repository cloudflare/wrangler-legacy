#![allow(clippy::redundant_closure)]

use std::env;
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
mod worker;

use crate::settings::project::ProjectType;
use terminal::emoji;

fn main() -> Result<(), failure::Error> {
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

    let matches = App::new(format!("{}{} wrangler", emoji::WORKER, emoji::SPARKLES))
        .version(env!("CARGO_PKG_VERSION"))
        .author("ashley g williams <ashley666ashley@gmail.com>")
        .setting(AppSettings::ArgRequiredElseHelp)
        .setting(AppSettings::DeriveDisplayOrder)
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
                )
            ),
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
                ),
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
                ))
                .arg(
                    Arg::with_name("email")
                        .help("the email address associated with your Cloudflare account")
                        .index(1)
                        .required(true),
                )
                .arg(
                    Arg::with_name("api-key")
                        .help("your Cloudflare API key")
                        .index(2)
                        .required(true),
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
                        .index(1)
                        .required(true),
                ),
        )
        .subcommand(SubCommand::with_name("whoami").about(&*format!(
            "{} Retrieve your user info and test your auth config",
            emoji::SLEUTH
        )))
        .get_matches();

    if let Some(matches) = matches.subcommand_matches("config") {
        let email = matches
            .value_of("email")
            .expect("An email address must be provided.");
        let api_key = matches
            .value_of("api-key")
            .expect("An API key must be provided.");

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

        let method = HTTPMethod::from_str(matches.value_of("method").unwrap_or("get"));

        let body = match matches.value_of("body") {
            Some(s) => Some(s.to_string()),
            None => None,
        };

        commands::preview(&project, method, body)?;
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
    }
    Ok(())
}
