mod server;
mod setup;
mod watch;

use setup::{upload, Session};
use watch::watch_for_changes;

use crate::commands::dev::{socket, ServerConfig};
use crate::settings::global_user::GlobalUser;
use crate::settings::toml::{DeployConfig, Target};

use tokio::runtime::Runtime as TokioRuntime;

use std::sync::{Arc, Mutex};
use std::thread;

pub fn dev(
    target: Target,
    user: GlobalUser,
    server_config: ServerConfig,
    deploy_config: DeployConfig,
    http: bool,
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

    let preview_token = Arc::new(Mutex::new(preview_token));

    {
        let preview_token = preview_token.clone();
        let session_token = session.preview_token.clone();

        thread::spawn(move || {
            watch_for_changes(
                target,
                &deploy_config,
                &user,
                Arc::clone(&preview_token),
                session_token,
                verbose,
            )
        });
    }

    let mut runtime = TokioRuntime::new()?;
    runtime.block_on(async {
        let devtools_listener = tokio::spawn(socket::listen(session.clone().websocket_url));
        if http {
            start_http(server_config, preview_token, session.clone()).await?;
        } else {
            start_https(server_config, preview_token, session.clone()).await;
        }

        //let res = tokio::try_join!(async { devtools_listener.await? }, async { server.await? });
        devtools_listener.await?
    })
}

async fn start_http(
    server_config: ServerConfig,
    preview_token: Arc<Mutex<String>>,
    session: Session,
) -> Result<(), failure::Error> {
    let server = tokio::spawn(server::http(
        server_config,
        Arc::clone(&preview_token),
        session.host,
    ));

    server.await?
}

async fn start_https(
    server_config: ServerConfig,
    preview_token: Arc<Mutex<String>>,
    session: Session,
) {
    let mut server = tokio::spawn(server::https(
        server_config.clone(),
        Arc::clone(&preview_token),
        session.host.clone(),
    ));

    while server.await.is_ok() {
        server = tokio::spawn(server::https(
            server_config.clone(),
            Arc::clone(&preview_token),
            session.host.clone(),
        ));
    }
}
