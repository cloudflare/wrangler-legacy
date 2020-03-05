mod server;
mod setup;

use server::serve;

use crate::commands;
use crate::commands::dev::ServerConfig;
use crate::settings::global_user::GlobalUser;
use crate::settings::toml::{DeployConfig, Target};

use tokio::runtime::Runtime as TokioRuntime;

pub fn dev(
    target: Target,
    deploy_config: DeployConfig,
    user: GlobalUser,
    server_config: ServerConfig,
    verbose: bool,
) -> Result<(), failure::Error> {
    commands::build(&target)?;
    let preview_token = setup::get_preview_token(&deploy_config, &user)?;
    let host = match deploy_config.clone() {
        DeployConfig::Zoned(config) => "theharnishes.com",
        DeployConfig::Zoneless(config) => "worker.avery.workers.dev",
    }
    .to_string();
    let mut target = target.clone();
    // TODO: replace asset manifest parameter
    let preview_token = setup::upload(&mut target, None, &deploy_config, &user, preview_token)?;
    let server = serve(server_config, preview_token, host);
    let mut runtime = TokioRuntime::new()?;
    runtime.block_on(server)
}
