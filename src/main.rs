use clap::{App, Arg, SubCommand};
use settings::Settings;

mod commands;
mod settings;

fn main() -> Result<(), failure::Error> {
    let settings = Settings::new()?;

    let matches = App::new("👷‍♀️🧡 ☁️ ✨ wrangler")
        .version("0.1.0")
        .author("ashley g williams <ashley666ashley@gmail.com>")
        .subcommand(
            SubCommand::with_name("generate").about("👯 Generate a new wasm worker project"),
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
            SubCommand::with_name("whoami")
                .about("🕵️‍♀️ Retrieve your user info and test your auth config"),
        )
        .get_matches();

    if let Some(matches) = matches.subcommand_matches("publish") {
        let zone_id = matches
            .value_of("zone_id")
            .expect("A zone ID must be provided.");

        commands::build()?;
        commands::publish(zone_id, settings.clone())?;
    }

    if matches.subcommand_matches("whoami").is_some() {
        commands::whoami(settings)?;
    }

    if matches.subcommand_matches("generate").is_some() {
        commands::generate()?;
    }

    if matches.subcommand_matches("build").is_some() {
        commands::build()?;
    }
    Ok(())
}
