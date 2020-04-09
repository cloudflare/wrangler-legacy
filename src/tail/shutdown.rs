use tokio::sync::oneshot::{channel, Receiver, Sender};

pub enum Signal {
    Continue,
    ShutDown,
}

#[derive(Default)]
pub struct ShutdownHandler {
    txs: Vec<Sender<()>>,
}

impl ShutdownHandler {
    pub fn new() -> ShutdownHandler {
        ShutdownHandler::default()
    }

    pub fn subscribe(&mut self) -> Receiver<()> {
        let (tx, rx) = channel();
        self.txs.push(tx);

        rx
    }

/// handle_sigint waits on a ctrl_c from the system and sends messages to each registered
/// transmitter when it is received.
    pub async fn run(self) -> Result<(), failure::Error> {
        tokio::signal::ctrl_c().await?;
        for tx in self.txs {
            // if `tx.send()` returns an error, it is because the receiver has gone out of scope,
            // likely due to the task returning early for some reason, in which case we don't need
            // to tell that task to shut down because it already has.
            tx.send(()).ok();
        }

        Ok(())
    }
}
