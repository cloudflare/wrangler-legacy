use std::process::Command;

use log::info;

pub mod build;
pub mod config;
pub mod generate;
pub mod init;
pub mod kv;
pub mod publish;
pub mod subdomain;
pub mod whoami;

pub use self::config::global_config;
pub use build::build;
pub use build::watch_and_build;
pub use generate::{generate, generate_site};
pub use init::init;
pub use publish::preview::preview;
pub use publish::preview::HTTPMethod;
pub use publish::publish;
pub use subdomain::subdomain;
pub use whoami::whoami;

/// Run the given command and return its stdout.
pub fn run(mut command: Command, command_name: &str) -> Result<(), failure::Error> {
    info!("Running {:?}", command);

    let status = command.status()?;

    if status.success() {
        Ok(())
    } else {
        failure::bail!(
            "failed to execute `{}`: exited with {}",
            command_name,
            status
        )
    }
}
