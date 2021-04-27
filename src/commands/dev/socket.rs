use std::time::Duration;

use chrome_devtools as protocol;

use futures_util::future::TryFutureExt;
use futures_util::sink::SinkExt;
use futures_util::stream::{SplitStream, StreamExt};
use tokio_stream::wrappers::UnboundedReceiverStream;

use crate::terminal::message::{Message, StdErr, StdOut};
use tokio::net::TcpStream;
use tokio::sync::mpsc;
use tokio::time::sleep;

use tokio_tungstenite::{connect_async, tungstenite, MaybeTlsStream, WebSocketStream};

use anyhow::{anyhow, Result};
use url::Url;

const KEEP_ALIVE_INTERVAL: u64 = 10;

/// connect to a Workers runtime WebSocket emitting the Chrome Devtools Protocol
/// parse all console messages, and print them to stdout
pub async fn listen(socket_url: Url) -> Result<()> {
    // we loop here so we can issue a reconnect when something
    // goes wrong with the websocket connection
    loop {
        let ws_stream = connect_retry(&socket_url).await;

        let (mut write, read) = ws_stream.split();

        // console.log messages are in the Runtime domain
        // we must signal that we want to receive messages from the Runtime domain
        // before they will be sent
        let enable_runtime = protocol::runtime::SendMethod::Enable(1.into());
        let enable_runtime = serde_json::to_string(&enable_runtime)?;
        let enable_runtime = tungstenite::protocol::Message::Text(enable_runtime);
        write.send(enable_runtime).await?;

        // if left unattended, the preview service will kill the socket
        // that emits console messages
        // send a keep alive message every so often in the background
        let (keep_alive_tx, keep_alive_rx) = mpsc::unbounded_channel();

        // every 10 seconds, send a keep alive message on the channel
        let heartbeat = keep_alive(keep_alive_tx);

        // when the keep alive channel receives a message from the
        // heartbeat future, write it to the websocket
        let keep_alive_to_ws = UnboundedReceiverStream::new(keep_alive_rx)
            .map(Ok)
            .forward(write)
            .map_err(Into::into);

        // parse all incoming messages and print them to stdout
        let printer = print_ws_messages(read);

        // run the heartbeat and message printer in parallel
        if tokio::try_join!(heartbeat, keep_alive_to_ws, printer).is_ok() {
            break Ok(());
        } else {
        }
    }
}

// Endlessly retry connecting to the chrome devtools instance with exponential backoff.
// The backoff maxes out at 60 seconds.
async fn connect_retry(socket_url: &Url) -> WebSocketStream<MaybeTlsStream<TcpStream>> {
    let mut wait_seconds = 2;
    let maximum_wait_seconds = 60;
    let mut failed = false;
    loop {
        match connect_async(socket_url).await {
            Ok((ws_stream, _)) => {
                if failed {
                    // only report success if there was a failure, otherwise be quiet about it
                    StdErr::success("Connected!");
                }
                return ws_stream;
            }
            Err(e) => {
                failed = true;
                StdErr::warn(&format!("Failed to connect to devtools instance: {}", e));
                StdErr::warn(&format!(
                    "Will retry connection in {} seconds",
                    wait_seconds
                ));
                sleep(Duration::from_secs(wait_seconds)).await;
                wait_seconds = wait_seconds.pow(2);
                if wait_seconds > maximum_wait_seconds {
                    // max out at 60 seconds
                    wait_seconds = maximum_wait_seconds;
                }
                StdErr::working("Retrying...");
            }
        }
    }
}

async fn print_ws_messages(
    mut read: SplitStream<WebSocketStream<MaybeTlsStream<TcpStream>>>,
) -> Result<()> {
    while let Some(message) = read.next().await {
        match message {
            Ok(message) => {
                let message_text = message.into_text().unwrap();
                log::info!("{}", &message_text);

                let parsed_message: Result<protocol::Runtime> = serde_json::from_str(&message_text)
                    .map_err(|e| anyhow!("this event could not be parsed:\n{}", e));

                if let Ok(protocol::Runtime::Event(event)) = parsed_message {
                    // Try to parse json to pretty print, otherwise just print string
                    let json_parse: Result<serde_json::Value, serde_json::Error> =
                        serde_json::from_str(&*event.to_string());
                    if let Ok(json) = json_parse {
                        if let Ok(json_str) = serde_json::to_string_pretty(&json) {
                            println!("{}", json_str);
                        } else {
                            StdOut::message(&format!("{:?}", event.to_string()));
                        }
                    } else {
                        println!("{}", event);
                    }
                }
            }
            Err(error) => return Err(error.into()),
        }
    }
    Ok(())
}

async fn keep_alive(tx: mpsc::UnboundedSender<tungstenite::protocol::Message>) -> Result<()> {
    let duration = Duration::from_millis(1000 * KEEP_ALIVE_INTERVAL);
    let mut delay = sleep(duration);

    // this is set to 2 because we have already sent an id of 1 to enable the runtime
    // eventually this logic should be moved to the chrome-devtools-rs library
    let mut id = 2;

    loop {
        delay.await;
        let keep_alive_message = protocol::runtime::SendMethod::GetIsolateId(id.into());
        let keep_alive_message = serde_json::to_string(&keep_alive_message)
            .expect("Could not convert keep alive message to JSON");
        let keep_alive_message = tungstenite::protocol::Message::Text(keep_alive_message);
        tx.send(keep_alive_message).unwrap();
        id += 1;
        delay = sleep(duration);
    }
}
