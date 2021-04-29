use std::path::PathBuf;
use std::process::Stdio;
use std::str;

use tokio::process::Child;
use tokio::process::Command;
use tokio::sync::oneshot::Receiver;

use anyhow::Result;

pub struct Tunnel {
    child: Child,
}

/// Tunnel wraps a child process that runs cloudflared and forwards requests from the Trace Worker
/// in the runtime to our local LogServer instance. We wrap it in a struct primarily to hold the
/// state of the child process so that upon receipt of a SIGINT message we can more swiftly kill it
/// and wait on its output; otherwise we leave an orphaned process when wrangler exits and this
/// causes problems if it still exists the next time we start up a tail.
impl Tunnel {
    pub fn new(tunnel_port: u16, metrics_port: u16, verbose: bool) -> Result<Tunnel> {
        let tool_name = PathBuf::from("cloudflared");
        // TODO: Finally get cloudflared release binaries distributed on GitHub so we could
        // simply uncomment the line below.
        // let binary_path = install::install(tool_name, "cloudflare")?.binary(tool_name)?;

        let tunnel_url = format!("localhost:{}", tunnel_port);
        let metrics_url = format!("localhost:{}", metrics_port);
        let args = ["tunnel", "--url", &tunnel_url, "--metrics", &metrics_url];

        let mut command = command(&args, &tool_name, verbose);
        let command_name = format!("{:?}", command);

        let child = command
            .spawn()
            .unwrap_or_else(|_| panic!("{} failed to spawn", command_name));

        Ok(Tunnel { child })
    }

    pub async fn run(self, shutdown_rx: Receiver<()>) -> Result<()> {
        shutdown_rx.await?;
        self.shutdown().await
    }

    /// shutdown is relatively simple, it sends a second `kill` signal to the child process,
    /// short-circuiting cloudflared's "graceful shutdown" period. this approach has been endorsed
    /// by the team who maintains cloudflared as safe practice.
    pub async fn shutdown(mut self) -> Result<()> {
        if let Err(e) = self.child.kill().await {
            let msg = if let Some(pid) = self.child.id() {
                format!("failed to kill cloudflared: {}\ncloudflared will eventually exit, or you can explicitly kill it by running `kill {}`", e, pid)
            } else {
                format!(
                    "failed to kill cloudflared: {}\ncloudflared will eventually exit.",
                    e
                )
            };
            anyhow::bail!(msg)
        } else {
            self.child.wait_with_output().await?;

            Ok(())
        }
    }
}

// TODO: let's not clumsily copy this from commands/build/mod.rs
// We definitely want to keep the check for RUST_LOG=info below so we avoid
// spamming user terminal with default cloudflared output (which is pretty darn sizable.)
pub fn command(args: &[&str], binary_path: &PathBuf, verbose: bool) -> Command {
    let mut c = if cfg!(target_os = "windows") {
        let mut c = Command::new("cmd");
        c.arg("/C");
        c.arg(binary_path);
        c
    } else {
        Command::new(binary_path)
    };

    c.args(args);
    // Let user read cloudflared process logs iff RUST_LOG=info.
    if !verbose {
        c.stderr(Stdio::null());
        c.stdout(Stdio::null());
    }

    c
}
