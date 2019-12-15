use super::cli_prelude::*;

/// Defines the `subdomain` sub-command entry point.
pub fn sub_command() -> App {
    #[cfg(target_os = "macos")]
    const ABOUT: &str = "👷  Configure your workers.dev subdomain";
    #[cfg(not(target_os = "macos"))]
    const ABOUT: &str = "Configure your workers.dev subdomain";

    SubCommand::with_name("subdomain").about(ABOUT).arg(
        Arg::with_name("name")
            .help("the subdomain on workers.dev you'd like to reserve")
            .index(1),
    )
}
