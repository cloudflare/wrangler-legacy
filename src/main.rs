use std::str::FromStr;

use cache::get_wrangler_cache;
use clap::{App, Arg, SubCommand};
use commands::HTTPMethod;

use log::info;

mod cache;
mod commands;
mod emoji;
mod install;
mod installer;
mod user;
mod wranglerjs;

use user::User;

fn main() -> Result<(), failure::Error> {
    env_logger::init();
    let cache = get_wrangler_cache()?;

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
        .subcommand(
            SubCommand::with_name("generate")
                .about(&*format!(
                    "{} Generate a new wasm worker project",
                    emoji::DANCERS
                ))
                .arg(
                    Arg::with_name("name")
                        .help("the name of your worker! defaults to 'wasm-worker'")
                        .index(1),
                )
                .arg(
                    Arg::with_name("template")
                        .help("a link to a github template! defaults to cloudflare/worker-template")
                        .index(2),
                ),
        )
        .subcommand(
            SubCommand::with_name("preview")
                .about(&*format!(
                    "{} Publish your code temporarily on cloudflareworkers.com",
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
            SubCommand::with_name("build")
                .about(&*format!("{} Build your wasm with wasm-pack", emoji::CRAB)),
        )
        .subcommand(SubCommand::with_name("publish").about(&*format!(
            "{} Push your worker to the orange cloud",
            emoji::UP
        )))
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
        .subcommand(SubCommand::with_name("whoami").about(&*format!(
            "{} Retrieve your user info and test your auth config",
            emoji::SLEUTH
        )))
        .get_matches();

    if matches.subcommand_matches("config").is_some()
        || matches.subcommand_matches("generate").is_some()
    {
        if let Some(matches) = matches.subcommand_matches("config") {
            let email = matches
                .value_of("email")
                .expect("An email address must be provided.");
            let api_key = matches
                .value_of("api-key")
                .expect("An API key must be provided.");
            commands::global_config(email, api_key)?;
        }

        if let Some(matches) = matches.subcommand_matches("generate") {
            let name = matches.value_of("name").unwrap_or("wasm-worker");
            let template = matches
                .value_of("template")
                .unwrap_or("https://github.com/cloudflare/worker-template");
            info!(
                "Generate command called with template {}, and name {}",
                template, name
            );
            commands::generate(name, template, &cache)?;
        }
    } else {
        info!("Getting user and project settings");
        let user = User::new()?;

        if matches.subcommand_matches("whoami").is_some() {
            commands::whoami(&user);
        }

        if matches.subcommand_matches("build").is_some() {
            commands::build(&cache, &user.settings.project.project_type)?;
        }

        if let Some(matches) = matches.subcommand_matches("preview") {
            let method = HTTPMethod::from_str(matches.value_of("method").unwrap_or("get"));

            let body = match matches.value_of("body") {
                Some(s) => Some(s.to_string()),
                None => None,
            };

            commands::build(&cache, &user.settings.project.project_type)?;
            commands::preview(method, body)?;
        }

        if matches.subcommand_matches("publish").is_some() {
            commands::build(&cache, &user.settings.project.project_type)?;
            commands::publish(user)?;
        }
    }
    Ok(())
}
