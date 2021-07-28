use super::Cli;
use crate::commands;
use crate::commands::tail::filter::*;
use crate::commands::tail::websocket::{TailFormat, TailOptions};
use crate::settings::{global_user::GlobalUser, toml::Manifest};

use anyhow::Result;
use url::Url;

pub fn tail(
    name: Option<String>,
    url: Option<Url>,
    format: TailFormat,
    once: bool,
    sampling_rate: f64,
    outcomes: Vec<String>,
    methods: Vec<reqwest::Method>,
    headers: Vec<String>,
    client_ips: Vec<String>,
    search: Option<String>,
    cli_params: &Cli,
) -> Result<()> {
    let user = GlobalUser::new()?;

    // FIXME: If `name` is defined, allow the command to be run outside a `wrangler.toml` directory.
    let manifest = Manifest::new(&cli_params.config)?;
    let target = manifest.get_target(cli_params.environment.as_deref(), false)?;
    let account_id = target.account_id.load()?.to_string();
    let script_name = name.unwrap_or(target.name);

    let mut filters: Vec<Box<dyn TraceFilter>> = vec![];
    if !outcomes.is_empty() {
        filters.push(Box::new(OutcomeFilter::from(outcomes)));
    }
    if !methods.is_empty() {
        filters.push(Box::new(MethodFilter::from(methods)));
    }
    if !client_ips.is_empty() {
        filters.push(Box::new(ClientIpFilter::from(client_ips)));
    }
    for header in headers.into_iter() {
        filters.push(Box::new(HeaderFilter::from(header)))
    }
    if let Some(query) = search {
        filters.push(Box::new(QueryFilter::from(query)));
    };
    if sampling_rate < 1.0 {
        filters.push(Box::new(SamplingRateFilter::from(sampling_rate))); // Should always be last
    };

    let options = TailOptions {
        format,
        once,
        filters,
    };
    let tail = commands::tail::run(user, account_id, script_name, url, options);

    tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap()
        .block_on(tail)
}
