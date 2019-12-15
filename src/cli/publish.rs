use super::cli_prelude::*;

/// Defines the `publish` sub-command entry point.
pub fn sub_command() -> App {
    #[cfg(target_os = "macos")]
    const ABOUT: &str = "🆙  Publish your worker to the orange cloud";
    #[cfg(not(target_os = "macos"))]
    const ABOUT: &str = "Publish your worker to the orange cloud";

    SubCommand::with_name("publish")
        .about(ABOUT)
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
