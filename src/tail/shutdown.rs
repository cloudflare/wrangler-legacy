use anyhow::Result;
use tokio::sync::oneshot::{channel, Receiver, Sender};

pub struct ShutdownHandler {
    txs: Vec<Sender<()>>,
}

impl ShutdownHandler {
    pub fn new() -> ShutdownHandler {
        ShutdownHandler { txs: Vec::new() }
    }

    pub fn subscribe(&mut self) -> Receiver<()> {
        let (tx, rx) = channel();
        self.txs.push(tx);

        rx
    }

    /// ShutdownHandler waits on a ctrl_c from the system, or a short circuit command from the top
    /// level error handler, and sends messages to each registered transmitter when it is received.
    pub async fn run(self, short_circuit: Receiver<()>) -> Result<()> {
        tokio::select! {
            _ = tokio::signal::ctrl_c() => {}
            _ = short_circuit => {}
        }

        eprintln!("Closing tail session...");
        for tx in self.txs {
            // if `tx.send()` returns an error, it is because the receiver has gone out of scope,
            // likely due to the task returning early for some reason, in which case we don't need
            // to tell that task to shut down because it already has.
            tx.send(()).ok();
        }

        Ok(())
    }
}
