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
) -> Result<(), failure::Error> {
    commands::build(&target)?;
    let (preview_token, host) = setup::init(&deploy_config, &user)?;
    let mut target = target.clone();
    // TODO: replace asset manifest parameter
    let preview_token = setup::upload(&mut target, None, &deploy_config, &user, preview_token)?;
    let server = serve(server_config, preview_token, host);
    let mut runtime = TokioRuntime::new()?;
    runtime.block_on(server)
}
