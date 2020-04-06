mod host;
mod session;
mod tunnel;

use host::Host;
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
            // Create three one-shot channels for handling ctrl-c
            let (log_tx, log_rx) = oneshot::channel();
            let (session_tx, session_rx) = tokio::sync::oneshot::channel();
            let (tunnel_tx, tunnel_rx) = tokio::sync::oneshot::channel();

            // Pass the three transmitters to a newly spawned sigint handler
            let txs = vec![log_tx, tunnel_tx, session_tx];
            let sigint_handle = tokio::spawn(handle_sigint(txs));

            // Spin up a local http server to receive logs
            let log_host = Host::new(log_rx);
            let host_handle = tokio::spawn(log_host.run());

            // Spin up a new cloudflared tunnel to connect trace worker to local server
            let tunnel_process = Tunnel::new()?;
            let tunnel_handle = tokio::spawn(tunnel_process.run(tunnel_rx));

            // Register the tail with the Workers API and send periodic heartbeats
            let session_handle = tokio::spawn(Session::run(target, user, session_rx));

            let res = tokio::try_join!(sigint_handle, host_handle, session_handle, tunnel_handle,);

            match res {
                Ok(_) => Ok(()),
                Err(e) => failure::bail!(e),
            }
        })
    }
}

async fn handle_sigint(txs: Vec<oneshot::Sender<()>>) -> Result<(), failure::Error> {
    tokio::signal::ctrl_c().await?;
    for tx in txs {
        if let Err(e) = tx.send(()) {
            eprintln!("failed to transmit to channel {:?}", e);
        }
    }

    Ok(())
}
