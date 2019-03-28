use std::str::FromStr;

use binary_install::Cache;
use clap::{App, Arg, SubCommand};
use commands::HTTPMethod;
use settings::Settings;

mod commands;
mod install;
mod settings;

fn main() -> Result<(), failure::Error> {
    env_logger::init();
    let cache = Cache::new("wrangler")?;

    let matches = App::new("👷‍♀️🧡☁️ ✨ wrangler")
        .version("0.1.0")
        .author("ashley g williams <ashley666ashley@gmail.com>")
        .subcommand(
            SubCommand::with_name("generate")
                .about("👯 Generate a new wasm worker project")
                .arg(
                    Arg::with_name("name")
                        .help("the name of your worker! defaults to 'wasm-worker'")
                        .index(1),
                ),
        )
        .subcommand(
            SubCommand::with_name("preview")
                .about("🔬Publish your code temporarily on cloudflareworkers.com")
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
            SubCommand::with_name("build").about("🦀⚙️ Build your wasm with wasm-pack"),
        )
        .subcommand(
            SubCommand::with_name("publish")
                .about("☁️ 🆙 Push your worker to the orange cloud")
                .arg(
                    Arg::with_name("zone_id")
                        .help("the ID of the zone to publish the worker to")
                        .index(1)
                        .required(true),
                ),
        )
        .subcommand(
            SubCommand::with_name("config")
                .about("🕵️‍♀️ Setup wrangler with your Cloudflare account")
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
            SubCommand::with_name("whoami")
                .about("🕵️‍♀️ Retrieve your user info and test your auth config"),
        )
        .get_matches();

    if matches.subcommand_matches("publish").is_none()
        && matches.subcommand_matches("whoami").is_none()
    {
        if let Some(matches) = matches.subcommand_matches("config") {
            let email = matches
                .value_of("email")
                .expect("An email address must be provided.");
            let api_key = matches
                .value_of("api-key")
                .expect("An API key must be provided.");
            commands::config(email, api_key)?;
        }

        if let Some(matches) = matches.subcommand_matches("preview") {
            let method = HTTPMethod::from_str(matches.value_of("method").unwrap_or("get"));

            let body = match matches.value_of("body") {
                Some(s) => Some(s.to_string()),
                None => None,
            };

            commands::build(&cache)?;
            commands::preview(method, body)?;
        }

        if let Some(matches) = matches.subcommand_matches("generate") {
            let name = matches.value_of("name").unwrap_or("wasm-worker");
            commands::generate(name, &cache)?;
        }

        if matches.subcommand_matches("build").is_some() {
            commands::build(&cache)?;
        }
    } else {
        let settings = Settings::new()
            .expect("🚧 Whoops! You aren't configured yet. Run `wrangler config`! 🚧");

        if let Some(matches) = matches.subcommand_matches("publish") {
            let zone_id = matches
                .value_of("zone_id")
                .expect("A zone ID must be provided.");

            commands::build(&cache)?;
            commands::publish(zone_id, settings.clone())?;
        }

        if matches.subcommand_matches("whoami").is_some() {
            commands::whoami(settings)?;
        }
    }
    Ok(())
}
