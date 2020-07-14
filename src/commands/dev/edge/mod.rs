mod server;
mod setup;

use server::serve;
use setup::Init;

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
    let init = Init::new(&target, &deploy_config, &user)?;
    let mut target = target;

    // TODO: replace asset manifest parameter
    let preview_token = setup::upload(
        &mut target,
        &deploy_config,
        &user,
        init.preview_token,
        verbose,
    )?;
    let server = serve(server_config, preview_token, init.host);
    let mut runtime = TokioRuntime::new()?;
    runtime.block_on(server)
}
