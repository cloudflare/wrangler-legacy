#![cfg_attr(feature = "strict", deny(warnings))]

extern crate text_io;
extern crate tokio;

use std::env;

use wrangler::cli::{exec, Cli, Command};
use wrangler::installer;
use wrangler::version::check_for_updates;

use anyhow::Result;
use structopt::StructOpt;

fn main() -> Result<()> {
    env_logger::init();

    StdErr::deprecation_warning(&format!(
        r#"{} {} {}
You are using deprecated Wrangler 1 (v{}).
Please upgrade to v2 ({}).
Detailed migration instructions can be found here:
{}
"#,
        emoji::NO_ENTRY,
        styles::warning("DEPRECATION"),
        emoji::NO_ENTRY,
        env!("CARGO_PKG_VERSION"),
        styles::url("https://github.com/cloudflare/wrangler2"),
        styles::url("https://developers.cloudflare.com/workers/wrangler/migration/migrating-from-wrangler-1/")
    ));

    if let Ok(me) = env::current_exe() {
        // If we're actually running as the installer then execute our
        // self-installation, otherwise just continue as usual.
        if me
            .file_stem()
            .and_then(|s| s.to_str())
            .expect("executable should have a filename")
            .starts_with("wrangler-init")
        {
            installer::install()?;
        }
    }
    run()?;
    check_for_updates();
    Ok(())
}

fn run() -> Result<()> {
    let cli = Cli::from_args();
    let cli_params = cli.clone();

    match cli.command {
        Command::Config { api_key, no_verify } => exec::configure(api_key, no_verify),
        Command::Generate {
            name,
            site,
            template,
            target_type,
        } => exec::generate(name, site, template, target_type),
        Command::Init {
            name,
            site,
            target_type,
        } => exec::init(name, site, target_type),
        Command::Build => exec::build(&cli_params),
        Command::Preview {
            method,
            url,
            body,
            watch,
            headless,
        } => exec::preview(method, url, body, watch, headless, &cli_params),
        Command::Dev {
            host,
            ip,
            port,
            local_protocol,
            upstream_protocol,
            inspect,
            unauthenticated,
        } => exec::dev(
            host,
            ip,
            port,
            local_protocol,
            upstream_protocol,
            &cli_params,
            inspect,
            unauthenticated,
        ),
        Command::Whoami => exec::whoami(),
        Command::Publish {
            release,
            output,
            migration,
        } => exec::publish(release, output, migration, &cli_params),
        Command::Subdomain { name } => exec::subdomain(name, &cli_params),
        Command::Route(route) => exec::route(route, &cli_params),
        Command::Secret(secret) => exec::secret(secret, &cli_params),
        Command::R2(r2) => exec::r2_bucket(r2, &cli_params),
        Command::KvNamespace(namespace) => exec::kv_namespace(namespace, &cli_params),
        Command::KvKey(key) => exec::kv_key(key, &cli_params),
        Command::KvBulk(bulk) => exec::kv_bulk(bulk, &cli_params),
        Command::Tail {
            name,
            url,
            format,
            once,
            sampling_rate,
            status,
            method,
            header,
            ip_address,
            search,
            ..
        } => exec::tail(
            name,
            url,
            format,
            once,
            sampling_rate,
            status,
            method,
            header,
            ip_address,
            search,
            &cli_params,
        ),
        Command::Login {
            scopes,
            scopes_list,
        } => exec::login(&scopes, scopes_list),
        Command::Logout => exec::logout(),
    }
}
