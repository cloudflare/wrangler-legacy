mod server;
mod setup;
mod watch;

use setup::{upload, Session};
use watch::watch_for_changes;

use crate::commands::dev::{socket, Protocol, ServerConfig};
use crate::deploy::DeployTarget;
use crate::settings::global_user::GlobalUser;
use crate::settings::toml::Target;
use anyhow::Result;

use tokio::runtime::Runtime as TokioRuntime;

use std::sync::{Arc, Mutex};
use std::thread;

pub fn dev(
    target: Target,
    user: GlobalUser,
    server_config: ServerConfig,
    deploy_target: DeployTarget,
    local_protocol: Protocol,
    upstream_protocol: Protocol,
    verbose: bool,
) -> Result<()> {
    let session = Session::new(&target, &user, &deploy_target)?;
    let mut target = target;

    let preview_token = upload(
        &mut target,
        &deploy_target,
        &user,
        session.preview_token.clone(),
        verbose,
    )?;

    let preview_token = Arc::new(Mutex::new(preview_token));

    {
        let preview_token = preview_token.clone();
        let session_token = session.preview_token.clone();

        thread::spawn(move || {
            watch_for_changes(
                target,
                &deploy_target,
                &user,
                Arc::clone(&preview_token),
                session_token,
                verbose,
            )
        });
    }

    let runtime = TokioRuntime::new()?;
    runtime.block_on(async {
        let devtools_listener = tokio::spawn(socket::listen(session.websocket_url));
        let server = match local_protocol {
            Protocol::Https => tokio::spawn(server::https(
                server_config.clone(),
                Arc::clone(&preview_token),
                session.host.clone(),
            )),
            Protocol::Http => tokio::spawn(server::http(
                server_config,
                Arc::clone(&preview_token),
                session.host,
                upstream_protocol,
            )),
        };

        let res = tokio::try_join!(async { devtools_listener.await? }, async { server.await? });
        match res {
            Ok(_) => Ok(()),
            Err(e) => Err(e),
        }
    })
}
