use super::Cli;
use crate::commands;
use crate::commands::tail::websocket::{TailFilters, TailFormat, TailOptions};
use crate::settings::{global_user::GlobalUser, toml::Manifest};

use anyhow::Result;
use url::Url;

pub fn tail(
    name: Option<String>,
    url: Option<Url>,
    format: TailFormat,
    once: bool,
    status: Vec<String>,
    http_status: Vec<u32>,
    method: Vec<String>,
    ip_address: Vec<String>,
    cli_params: &Cli,
) -> Result<()> {
    let user = GlobalUser::new()?;
    let manifest = Manifest::new(&cli_params.config)?;
    let target = manifest.get_target(cli_params.environment.as_deref(), false)?;
    let account_id = target.account_id.load()?.to_string();
    let script_name = name.unwrap_or(target.name);
    let options = TailOptions {
        format,
        once,
        filters: TailFilters {
            status,
            http_status,
            method,
            ip_address,
        },
    };

    let tail = commands::tail::run(user, account_id, script_name, url, options);
    tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap()
        .block_on(tail)
}
