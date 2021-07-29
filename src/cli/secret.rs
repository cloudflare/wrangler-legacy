use super::Cli;
use crate::commands;
use crate::settings::{global_user::GlobalUser, toml::Manifest};

use anyhow::Result;
use structopt::StructOpt;
#[derive(Debug, Clone, StructOpt)]
#[structopt(rename_all = "lower")]
pub enum Secret {
    /// Create or update a secret variable for a script
    Put {
        #[structopt(long, short = "n", index = 1)]
        name: String,
    },
    /// Delete a secret variable from a script
    Delete {
        #[structopt(long, short = "n", index = 1)]
        name: String,
    },
    /// List all secrets for a script
    List,
}

pub fn secret(secret: Secret, cli_params: &Cli) -> Result<()> {
    log::info!("Getting User settings");
    let user = GlobalUser::new()?;

    log::info!("Getting project settings");
    let manifest = Manifest::new(&cli_params.config)?;
    let target = manifest.get_target(cli_params.environment.as_deref(), false)?;
    match secret {
        Secret::Put { name } => commands::secret::create_secret(&name, &user, &target),
        Secret::Delete { name } => commands::secret::delete_secret(&name, &user, &target),
        Secret::List => commands::secret::list_secrets(&user, &target),
    }
}
