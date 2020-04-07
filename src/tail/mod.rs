/// The Tail feature for Wrangler is the Workers ecosystem answer to the need for live
/// log collection in production. When a user runs `wrangler tail`, several things happen:
///     1. A simple HTTP server (LogServer) is started and begins listening for requests on localhost:8080
///     2. An [Argo Tunnel](TODO) instance (Tunnel) is started using [cloudflared](TODO), exposing the
///        LogServer to the internet on a randomly generated URL.
///     3. Wrangler initiates a tail Session by making a request to the Workers API /tail endpoint,
///        providing the Tunnel URL as an argument.
///     4. The Workers API binds the URL to a [Trace Worker], and directs all `console` and
///        exception logging to the Trace Worker, which POSTs each batch of logs as a JSON
///        payload to the provided Tunnel URL.
///     5. Upon receipt. the LogServer prints the payload of each POST request to STDOUT.
mod log_server;
mod session;
mod tunnel;

use log_server::LogServer;
use session::Session;
use tunnel::Tunnel;

use tokio;
use tokio::runtime::Runtime as TokioRuntime;
use tokio::sync::oneshot;

use crate::settings::global_user::GlobalUser;
use crate::settings::toml::Target;

pub struct Tail;

impl Tail {
    pub fn run(target: Target, user: GlobalUser) -> Result<(), failure::Error> {
        let mut runtime = TokioRuntime::new()?;

        runtime.block_on(async {
            // Create three [one-shot](https://docs.rs/tokio/0.2.16/tokio/sync#oneshot-channel)
            // channels for handling ctrl-c. Each channel has two parts:
            // tx: Transmitter
            // rx: Receiver
            let (log_tx, log_rx) = oneshot::channel();
            let (session_tx, session_rx) = oneshot::channel();
            let (tunnel_tx, tunnel_rx) = oneshot::channel();

            // Pass the three transmitters to a newly spawned sigint handler
            let txs = vec![log_tx, tunnel_tx, session_tx];
            let listener = tokio::spawn(listen_for_sigint(txs));

            // Spin up a local http server to receive logs
            let log_server = tokio::spawn(LogServer::new(log_rx).run());

            // Spin up a new cloudflared tunnel to connect trace worker to local server
            let tunnel_process = Tunnel::new()?;
            let tunnel = tokio::spawn(tunnel_process.run(tunnel_rx));

            // Register the tail with the Workers API and send periodic heartbeats
            let session = tokio::spawn(Session::run(target, user, session_rx));

            let res = tokio::try_join!(listener, log_server, session, tunnel);

            match res {
                Ok(_) => Ok(()),
                Err(e) => failure::bail!(e),
            }
        })
    }
}

/// handle_sigint simply waits on a ctrl_c from the system and sends messages to each registered
/// transmitter when it is received.
async fn listen_for_sigint(txs: Vec<oneshot::Sender<()>>) -> Result<(), failure::Error> {
    tokio::signal::ctrl_c().await?;
    for tx in txs {
        // if send errors, it is because the receiver has been de-allocated, and that's okay
        // because we were just telling it to shut down anyway.
        tx.send(()).ok();
    }

    Ok(())
}
