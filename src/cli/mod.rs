// Wrangler Library Imports.
use wrangler::terminal::emoji;

mod build;
mod config;
mod generate;
mod init;
mod kv;
mod preview;
mod publish;
mod subdomain;
mod whoami;

use cli_prelude::*;

pub fn app() -> App {
    App::new(format!("{}{} wrangler", emoji::WORKER, emoji::SPARKLES))
        .version(env!("CARGO_PKG_VERSION"))
        .author("The Wrangler Team <wrangler@cloudflare.com>")
        .settings(&[
            AppSettings::ArgRequiredElseHelp,
            AppSettings::DeriveDisplayOrder,
            AppSettings::VersionlessSubcommands,
        ])
        .subcommands(kv::all_kv_subcommands())
        .subcommand(generate::sub_command())
        .subcommand(init::sub_command())
        .subcommand(build::sub_command())
        .subcommand(preview::sub_command())
        .subcommand(publish::sub_command())
        .subcommand(config::sub_command())
        .subcommand(subdomain::sub_command())
        .subcommand(whoami::sub_command())
}

pub mod cli_prelude;
