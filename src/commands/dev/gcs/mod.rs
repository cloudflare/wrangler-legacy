mod headers;
mod server;
mod setup;
mod watch;

use setup::{get_preview_id, get_session_id};
use watch::watch_for_changes;

use crate::commands::dev::{socket, Protocol, ServerConfig};
use crate::settings::toml::Target;

use anyhow::Result;
use std::sync::{Arc, Mutex};
use std::thread;
use tokio::runtime::Runtime as TokioRuntime;
use url::Url;

/// spin up a local server that routes requests to the preview service
/// that has a Cloudflare Workers runtime without access to zone-specific features
pub fn dev(
    target: Target,
    server_config: ServerConfig,
    local_protocol: Protocol,
    verbose: bool,
    inspect: bool,
) -> Result<()> {
    println!("unauthenticated");

    // setup the session
    let session_id = get_session_id()?;
    let inspect = if inspect {
        Some(target.name.clone())
    } else {
        None
    };

    // upload the initial script
    let preview_id = get_preview_id(
        target.clone(),
        // there is no user for unauthenticated dev
        None,
        &server_config,
        &session_id,
        verbose,
    )?;

    // prewarm the request so `--inspect` works right away
    // note that this doesn't make a normal GET request, since that might affect the worker state
    if inspect.is_some() {
        let client = reqwest::blocking::Client::builder().build()?;
        client
            .post("https://prewarm.cloudflareworkers.com/")
            .header("CF-EW-Preview", &preview_id)
            .body("") // so reqwest will set the Content-Length header
            .send()?
            .error_for_status()?;
    }

    // the local server needs the preview ID to properly route
    // HTTP requests
    //
    // the file watcher updates the preview ID when there is a new
    // Worker
    //
    // Since these run on separate threads, we must stuff the
    // preview ID into an Arc<Mutex so that the server waits on the
    // file watcher to release the lock before routing a request
    let preview_id = Arc::new(Mutex::new(preview_id));
    // a new scope is created to satisfy the borrow checker
    {
        // we must clone each of these variables in order to
        // safely use them in another thread
        let session_id = session_id.clone();
        let preview_id = preview_id.clone();
        let server_config = server_config.clone();
        thread::spawn(move || {
            watch_for_changes(
                target,
                &server_config,
                Arc::clone(&preview_id),
                &session_id,
                verbose,
            )
        });
    }

    let socket_url = get_socket_url(&session_id)?;

    // in order to spawn futures we must create a tokio runtime
    let runtime = TokioRuntime::new()?;

    // and we must block the main thread on the completion of
    // said futures
    runtime.block_on(async {
        let devtools_listener = runtime.spawn(socket::listen(
            socket_url.clone(),
            server_config.clone(),
            inspect,
            None,
        ));

        let server = match local_protocol {
            Protocol::Https => runtime.spawn(server::https(
                server_config.clone(),
                Arc::clone(&preview_id),
            )),
            Protocol::Http => {
                runtime.spawn(server::http(server_config.clone(), Arc::clone(&preview_id)))
            }
        };

        let res = tokio::try_join!(async { devtools_listener.await? }, async { server.await? });
        match res {
            Ok(_) => Ok(()),
            Err(e) => Err(e),
        }
    })
}

fn get_socket_url(session_id: &str) -> Result<Url, url::ParseError> {
    Url::parse(&format!(
        "wss://cloudflareworkers.com/inspect/{}",
        session_id
    ))
}
