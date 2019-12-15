use super::cli_prelude::*;

/// Defines the `whoami` sub-command entry point.
pub fn sub_command() -> App {
    #[cfg(target_os = "macos")]
    const ABOUT: &str = "🕵️  Retrieve your user info and test your auth config";
    #[cfg(not(target_os = "macos"))]
    const ABOUT: &str = "Retrieve your user info and test your auth config";
    SubCommand::with_name("whoami").about(ABOUT)
}
