use std::time::Duration;

use chrome_devtools as protocol;

use futures::{future, pin_mut, StreamExt};
use futures_util::sink::SinkExt;

use tokio::time;
use tokio_tungstenite::connect_async;
use tungstenite::protocol::Message;

use url::Url;

const KEEP_ALIVE_INTERVAL: u64 = 10;

pub async fn listen(session_id: &str) -> Result<(), failure::Error> {
    let socket_url = format!("wss://rawhttp.cloudflareworkers.com/inspect/{}", session_id);
    let socket_url = Url::parse(&socket_url)?;

    let (ws_stream, _) = connect_async(socket_url)
        .await
        .expect("Failed to connect to devtools instance");

    let (mut write, read) = ws_stream.split();

    let enable_runtime = protocol::runtime::SendMethod::Enable(1.into());
    let enable_runtime = serde_json::to_string(&enable_runtime)?;
    let enable_runtime = Message::Text(enable_runtime);
    write.send(enable_runtime).await?;

    let (keep_alive_tx, keep_alive_rx) = futures::channel::mpsc::unbounded();
    tokio::spawn(keep_alive(keep_alive_tx));
    let keep_alive_to_ws = keep_alive_rx.map(Ok).forward(write);

    let print_ws_messages = {
        read.for_each(|message| async {
            let message = message.unwrap().into_text().unwrap();
            log::info!("{}", message);
            let message: Result<protocol::Runtime, failure::Error> = serde_json::from_str(&message)
                .map_err(|e| failure::format_err!("this event could not be parsed:\n{}", e));
            if let Ok(protocol::Runtime::Event(event)) = message {
                println!("{}", event);
            }
        })
    };
    pin_mut!(keep_alive_to_ws, print_ws_messages);
    future::select(keep_alive_to_ws, print_ws_messages).await;
    Ok(())
}

async fn keep_alive(tx: futures::channel::mpsc::UnboundedSender<Message>) -> ! {
    let duration = Duration::from_millis(1000 * KEEP_ALIVE_INTERVAL);
    let mut interval = time::interval(duration);

    // this is set to 2 because we have already sent an id of 1 to enable the runtime
    // eventually this logic should be moved to the chrome-devtools-rs library
    let mut id = 2;

    loop {
        interval.tick().await;
        let keep_alive_message = protocol::runtime::SendMethod::GetIsolateId(id.into());
        let keep_alive_message = serde_json::to_string(&keep_alive_message)
            .expect("Could not convert keep alive message to JSON");
        let keep_alive_message = Message::Text(keep_alive_message);
        tx.unbounded_send(keep_alive_message).unwrap();
        id += 1;
    }
}
