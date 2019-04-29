use std::str::FromStr;

use cache::get_wrangler_cache;
use clap::{App, Arg, SubCommand};
use commands::HTTPMethod;

mod cache;
mod commands;
mod install;
mod user;
mod wranglerjs;

use user::User;

fn main() -> Result<(), failure::Error> {
    env_logger::init();
    let cache = get_wrangler_cache()?;

    let matches = App::new("👷‍♀️🧡☁️ ✨ wrangler")
        .version(env!("CARGO_PKG_VERSION"))
        .author("ashley g williams <ashley666ashley@gmail.com>")
        .subcommand(
            SubCommand::with_name("generate")
                .about("👯 Generate a new wasm worker project")
                .arg(
                    Arg::with_name("name")
                        .help("the name of your worker! defaults to 'wasm-worker'")
                        .index(1),
                )
                .arg(
                    Arg::with_name("template")
                        .help("a link to a github template! defaultes to cloudflare/rustwasm-worker-template")
                        .index(2),
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
                    Arg::with_name("name")
                        .help("(optional) For multiscript users, provide a script name")
                        .index(1)
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
            commands::global_config(email, api_key)?;
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
            let template = matches
                .value_of("template")
                .unwrap_or("https://github.com/cloudflare/rustwasm-worker-template");
            commands::generate(name, template, &cache)?;
        }

        if matches.subcommand_matches("build").is_some() {
            commands::build(&cache)?;
        }
    } else {
        let user = User::new()?;

        if matches.subcommand_matches("whoami").is_some() {
            commands::whoami(&user);
        }

        if let Some(matches) = matches.subcommand_matches("publish") {
            let name = matches.value_of("name");

            commands::build(&cache)?;
            commands::publish(user, name)?;
        }
    }
    Ok(())
}
