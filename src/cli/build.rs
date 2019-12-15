use super::cli_prelude::*;

/// Defines the `build` sub-command entry point.
pub fn sub_command() -> App {
    #[cfg(target_os = "macos")]
    const ABOUT: &str = "🦀  Build your worker";
    #[cfg(not(target_os = "macos"))]
    const ABOUT: &str = "";

    SubCommand::with_name("build").about(ABOUT).arg(
        Arg::with_name("env")
            .help("environment to build")
            .short("e")
            .long("env")
            .takes_value(true),
    )
}
