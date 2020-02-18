mod headers;
mod server;
mod setup;
mod watch;

use server::serve;
use setup::{get_preview_id, get_session_id};
use watch::watch_for_changes;

use crate::commands;
use crate::commands::dev::{socket, ServerConfig};
use crate::settings::toml::Target;

use std::sync::{Arc, Mutex};
use std::thread;

use tokio::runtime::Runtime as TokioRuntime;
use url::Url;

pub fn dev(
    target: Target,
    server_config: ServerConfig,
    verbose: bool,
) -> Result<(), failure::Error> {
    commands::build(&target)?;
    let session_id = get_session_id()?;
    let preview_id = get_preview_id(
        target.clone(),
        None,
        &server_config,
        &session_id.clone(),
        verbose,
    )?;
    let preview_id = Arc::new(Mutex::new(preview_id));

    {
        let session_id = session_id.clone();
        let preview_id = preview_id.clone();
        let server_config = server_config.clone();
        thread::spawn(move || {
            watch_for_changes(
                target,
                None,
                &server_config,
                Arc::clone(&preview_id),
                &session_id,
                verbose,
            )
        });
    }
    let socket_url = format!("wss://rawhttp.cloudflareworkers.com/inspect/{}", session_id);
    let socket_url = Url::parse(&socket_url)?;
    let devtools_listener = socket::listen(socket_url);
    let server = serve(server_config, Arc::clone(&preview_id));

    let runners = futures::future::join(devtools_listener, server);
    let mut runtime = TokioRuntime::new()?;
    runtime.block_on(async {
        let (devtools_listener, server) = runners.await;
        devtools_listener?;
        server
    })
}
