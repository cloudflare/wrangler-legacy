/// `wrangler tail` allows Workers users to collect logs from their deployed Workers.
/// When a user runs `wrangler tail`, several things happen:
///     1. A simple HTTP server (LogServer) is started and begins listening for requests on localhost:8080
///     2. An [Argo Tunnel](https://developers.cloudflare.com/argo-tunnel/) instance (Tunnel) is started
///        using [cloudflared](https://developers.cloudflare.com/argo-tunnel/downloads/), exposing the
///        LogServer to the internet on a randomly generated URL.
///     3. Wrangler initiates a tail Session by making a request to the Workers API /tail endpoint,
///        providing the Tunnel URL as an argument.
///     4. The Workers API binds the URL to a [Trace Worker], and directs all `console` and
///        exception logging to the Trace Worker, which POSTs each batch of logs as a JSON
///        payload to the provided Tunnel URL.
///     5. Upon receipt, the LogServer prints the payload of each POST request to STDOUT.
mod log_server;
mod session;
mod shutdown;
mod tunnel;

use log_server::LogServer;
use session::Session;
use shutdown::ShutdownHandler;
use tunnel::Tunnel;

use anyhow::Result;
use console::style;
use tokio::runtime::Runtime as TokioRuntime;
use which::which;

use crate::settings::global_user::GlobalUser;
use crate::settings::toml::Target;
use crate::terminal::emoji;

pub struct Tail;

impl Tail {
    pub fn run(
        target: Target,
        user: GlobalUser,
        format: String,
        tunnel_port: u16,
        metrics_port: u16,
        verbose: bool,
    ) -> Result<()> {
        is_cloudflared_installed()?;
        print_startup_message(&target.name, tunnel_port, metrics_port);

        let runtime = TokioRuntime::new()?;

        runtime.block_on(async {
            // Create three [one-shot](https://docs.rs/tokio/0.2.16/tokio/sync#oneshot-channel)
            // channels for handling ctrl-c. Each channel has two parts:
            // tx: Transmitter
            // rx: Receiver
            let (tx, rx) = tokio::sync::oneshot::channel(); // shutdown short circuit
            let mut shutdown_handler = ShutdownHandler::new();
            let log_rx = shutdown_handler.subscribe();
            let session_rx = shutdown_handler.subscribe();
            let tunnel_rx = shutdown_handler.subscribe();

            let listener = tokio::spawn(shutdown_handler.run(rx));

            // Spin up a local http server to receive logs
            let log_server = tokio::spawn(LogServer::new(tunnel_port, log_rx, format).run());

            // Spin up a new cloudflared tunnel to connect trace worker to local server
            let tunnel_process = Tunnel::new(tunnel_port, metrics_port, verbose)?;
            let tunnel = tokio::spawn(tunnel_process.run(tunnel_rx));

            // Register the tail with the Workers API and send periodic heartbeats
            let session = tokio::spawn(Session::run(
                target,
                user,
                session_rx,
                tx,
                metrics_port,
                verbose,
            ));

            let res = tokio::try_join!(
                async { listener.await? },
                async { log_server.await? },
                async { session.await? },
                async { tunnel.await? }
            );

            match res {
                Ok(_) => Ok(()),
                Err(e) => Err(e),
            }
        })
    }
}

fn is_cloudflared_installed() -> Result<()> {
    // this can be removed once we automatically install cloudflared
    if which("cloudflared").is_err() {
        let install_url = style("https://developers.cloudflare.com/cloudflare-one/connections/connect-apps/install-and-setup/installation")
            .blue()
            .bold();
        anyhow::bail!("You must install cloudflared to use wrangler tail.\n\nInstallation instructions can be found here:\n{}", install_url);
    } else {
        Ok(())
    }
}

fn print_startup_message(worker_name: &str, tunnel_port: u16, metrics_port: u16) {
    // Note that we use eprintln!() throughout this module; this is because we want any
    // helpful output to not be mixed with actual log JSON output, so we use this macro
    // to print messages to stderr instead of stdout (where log output is printed).
    eprintln!(
        "{} Setting up log streaming from Worker script \"{}\". Using ports {} and {}.",
        emoji::TAIL,
        worker_name,
        tunnel_port,
        metrics_port,
    );
}
