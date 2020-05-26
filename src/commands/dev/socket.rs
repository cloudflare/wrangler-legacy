use std::time::Duration;

use chrome_devtools as protocol;

use futures_util::future::TryFutureExt;
use futures_util::sink::SinkExt;
use futures_util::stream::{SplitStream, StreamExt};

use tokio::net::TcpStream;
use tokio::sync::mpsc;
use tokio::time::delay_for;
use tokio_tls::TlsStream;
use tokio_tungstenite::stream::Stream;
use tokio_tungstenite::{connect_async, tungstenite::protocol::Message, WebSocketStream};

use url::Url;

const KEEP_ALIVE_INTERVAL: u64 = 10;

/// connect to a Workers runtime WebSocket emitting the Chrome Devtools Protocol
/// parse all console messages, and print them to stdout
pub async fn listen(session_id: String) -> Result<(), failure::Error> {
    let socket_url = format!("wss://rawhttp.cloudflareworkers.com/inspect/{}", session_id);
    let socket_url = Url::parse(&socket_url)?;

    // we loop here so we can issue a reconnect when something
    // goes wrong with the websocket connection
    loop {
        let (ws_stream, _) = connect_async(&socket_url)
            .await
            .expect("Failed to connect to devtools instance");

        let (mut write, read) = ws_stream.split();

        // console.log messages are in the Runtime domain
        // we must signal that we want to receive messages from the Runtime domain
        // before they will be sent
        let enable_runtime = protocol::runtime::SendMethod::Enable(1.into());
        let enable_runtime = serde_json::to_string(&enable_runtime)?;
        let enable_runtime = Message::Text(enable_runtime);
        write.send(enable_runtime).await?;

        // if left unattended, the preview service will kill the socket
        // that emits console messages
        // send a keep alive message every so often in the background
        let (keep_alive_tx, keep_alive_rx) = mpsc::unbounded_channel();

        // every 10 seconds, send a keep alive message on the channel
        let heartbeat = keep_alive(keep_alive_tx);

        // when the keep alive channel receives a message from the
        // heartbeat future, write it to the websocket
        let keep_alive_to_ws = keep_alive_rx.map(Ok).forward(write).map_err(Into::into);

        // parse all incoming messages and print them to stdout
        let printer = print_ws_messages(read);

        // run the heartbeat and message printer in parallel
        if tokio::try_join!(heartbeat, keep_alive_to_ws, printer).is_ok() {
            break Ok(());
        } else {
        }
    }
}

async fn print_ws_messages(
    mut read: SplitStream<WebSocketStream<Stream<TcpStream, TlsStream<TcpStream>>>>,
) -> Result<(), failure::Error> {
    while let Some(message) = read.next().await {
        match message {
            Ok(message) => {
                let message_text = message.into_text().unwrap();
                log::info!("{}", message_text);
                let parsed_message: Result<protocol::Runtime, failure::Error> =
                    serde_json::from_str(&message_text).map_err(|e| {
                        failure::format_err!("this event could not be parsed:\n{}", e)
                    });
                if let Ok(protocol::Runtime::Event(event)) = parsed_message {
                    println!("{}", event);
                }
            }
            Err(error) => return Err(error.into()),
        }
    }
    Ok(())
}

async fn keep_alive(tx: mpsc::UnboundedSender<Message>) -> Result<(), failure::Error> {
    let duration = Duration::from_millis(1000 * KEEP_ALIVE_INTERVAL);
    let mut delay = delay_for(duration);

    // this is set to 2 because we have already sent an id of 1 to enable the runtime
    // eventually this logic should be moved to the chrome-devtools-rs library
    let mut id = 2;

    loop {
        delay.await;
        let keep_alive_message = protocol::runtime::SendMethod::GetIsolateId(id.into());
        let keep_alive_message = serde_json::to_string(&keep_alive_message)
            .expect("Could not convert keep alive message to JSON");
        let keep_alive_message = Message::Text(keep_alive_message);
        tx.send(keep_alive_message).unwrap();
        id += 1;
        delay = delay_for(duration);
    }
}
