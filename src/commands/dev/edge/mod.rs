mod server;
mod setup;

use server::serve;
use setup::{upload, Session};

use crate::commands::dev::{socket, ServerConfig};
use crate::settings::global_user::GlobalUser;
use crate::settings::toml::{DeployConfig, Target};

use tokio::runtime::Runtime as TokioRuntime;

pub fn dev(
    target: Target,
    user: GlobalUser,
    server_config: ServerConfig,
    deploy_config: DeployConfig,
    verbose: bool,
) -> Result<(), failure::Error> {
    let session = Session::new(&target, &user, &deploy_config)?;
    let mut target = target;

    let preview_token = upload(
        &mut target,
        &deploy_config,
        &user,
        session.preview_token.clone(),
        verbose,
    )?;

    let mut runtime = TokioRuntime::new()?;
    runtime.block_on(async {
        let devtools_listener = tokio::spawn(socket::listen(session.websocket_url));
        let server = tokio::spawn(serve(server_config, preview_token, session.host));
        let res = tokio::try_join!(async { devtools_listener.await? }, async { server.await? });

        match res {
            Ok(_) => Ok(()),
            Err(e) => Err(e),
        }
    })
}
