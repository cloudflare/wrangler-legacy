use super::cli_prelude::*;

pub fn sub_command() -> App {
    // TODO: bring back emoji::UP.
    SubCommand::with_name("publish")
        .about("Publish your worker to the orange cloud")
        .arg(
            Arg::with_name("env")
                .help("environments to publish to")
                .short("e")
                .long("env")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("verbose")
                .long("verbose")
                .takes_value(false)
                .help("toggle verbose output"),
        )
        .arg(
            Arg::with_name("release")
                .long("release")
                .takes_value(false)
                .help("[deprecated] alias of wrangler publish"),
        )
}
