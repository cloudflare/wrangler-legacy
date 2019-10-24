use std::process::Command;

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
pub use generate::generate;
pub use init::init;
pub use publish::preview::preview;
pub use publish::preview::HTTPMethod;
pub use publish::publish;
use regex::Regex;
pub use subdomain::get_subdomain;
pub use subdomain::set_subdomain;
pub use whoami::whoami;

/// Run the given command and return its stdout.
pub fn run(mut command: Command, command_name: &str) -> Result<(), failure::Error> {
    log::info!("Running {:?}", command);

    let status = command.status()?;

    if !status.success() {
        failure::bail!(
            "tried running command:\n{}\nexited with {}",
            command_name.replace("\"", ""),
            status
        )
    }
    Ok(())
}

// Ensures that Worker name is valid.
pub fn validate_worker_name(name: &str) -> Result<(), failure::Error> {
    let re = Regex::new(r"^[a-z0-9_][a-z0-9-_]*$").unwrap();
    if !re.is_match(&name) {
        failure::bail!("Worker name \"{}\" invalid. Ensure that you only use lowercase letters, dashes, underscores, and numbers.", name)
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_can_detect_invalid_worker_name() {
        let invalid_names = vec!["mySite", "nicky.fun"];
        for name in invalid_names {
            assert!(validate_worker_name(name).is_err());
        }
    }

    #[test]
    fn it_can_detect_valid_worker_name() {
        let valid_names = vec!["my-blog", "blog123", "bloggyity_blog"];
        for name in valid_names {
            assert!(validate_worker_name(name).is_ok());
        }
    }
}
