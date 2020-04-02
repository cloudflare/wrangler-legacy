use std::path::PathBuf;
use std::process::Stdio;
use std::str;
use std::thread;
use std::time::Duration;

use log::log_enabled;
use log::Level::Info;
use tokio::process::Child;
use tokio::process::Command;

pub struct Tunnel {
    child: Child,
    command_name: String,
}

impl Tunnel {
    pub fn new() -> Result<Tunnel, failure::Error> {
        // TODO: remove sleep!! Can maybe use channel to signal from http server thread to argo tunnel
        // thread that the server is ready on port 8080 and prepared for the cloudflared CLI to open an
        // Argo Tunnel to it.
        thread::sleep(Duration::from_secs(5));

        let tool_name = PathBuf::from("cloudflared");
        // TODO: Finally get cloudflared release binaries distributed on GitHub so we could simply uncomment
        // the line below.
        // let binary_path = install::install(tool_name, "cloudflare")?.binary(tool_name)?;

        // TODO: allow user to pass in their own ports in case ports 8080 (used by cloudflared)
        // and 8081 (used by cloudflared metrics) are both already being used.
        let args = ["tunnel", "--metrics", "localhost:8081"];

        let mut command = command(&args, &tool_name);
        let command_name = format!("{:?}", command);

        let child = command
            .kill_on_drop(true)
            .spawn()
            .expect(&format!("{} failed to spawn", command_name));

        Ok(Tunnel {
            child,
            command_name,
        })
    }

    pub async fn run(self) -> Result<(), failure::Error> {
        let status = self.child.await?;

        if !status.success() {
            failure::bail!(
                "tried running command:\n{}\nexited with {}",
                self.command_name.replace("\"", ""),
                status
            )
        }

        Ok(())
    }
}

// TODO: let's not clumsily copy this from commands/build/mod.rs
// We definitely want to keep the check for RUST_LOG=info below so we avoid
// spamming user terminal with default cloudflared output (which is pretty darn sizable.)
pub fn command(args: &[&str], binary_path: &PathBuf) -> Command {
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
    if !log_enabled!(Info) {
        c.stderr(Stdio::null());
    }

    c
}
