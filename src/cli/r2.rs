use super::Cli;
use crate::commands;
use crate::settings::{global_user::GlobalUser, toml::Manifest};

use anyhow::Result;
use structopt::StructOpt;

#[derive(Debug, Clone, StructOpt)]
#[structopt(rename_all = "lower")]
pub enum R2 {
    /// Interact with your Workers R2 Buckets
    Bucket(Bucket),
}

#[derive(Debug, Clone, StructOpt)]
#[structopt(rename_all = "lower")]
pub enum Bucket {
    /// List existing buckets
    List,
    /// Create a new bucket
    Create {
        /// The name for your new bucket
        #[structopt(index = 1)]
        name: String,
    },
    /// Delete an existing bucket
    Delete {
        /// The name of the bucket to delete
        /// Note: bucket must be empty
        #[structopt(index = 1)]
        name: String,
    },
}

pub fn r2_bucket(r2: R2, cli_params: &Cli) -> Result<()> {
    let user = GlobalUser::new()?;
    let manifest = Manifest::new(&cli_params.config)?;
    let env = cli_params.environment.as_deref();

    match r2 {
        R2::Bucket(Bucket::List) => commands::r2::list(&manifest, env, &user),
        R2::Bucket(Bucket::Create { name }) => commands::r2::create(&manifest, env, &user, &name),
        R2::Bucket(Bucket::Delete { name }) => commands::r2::delete(&manifest, env, &user, &name),
    }
}
