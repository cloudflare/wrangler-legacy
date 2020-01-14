use std::time::Duration;

use chrome_devtools::events::DevtoolsEvent;

use console::style;

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

    let enable_runtime = Message::Text(
        r#"{
        "id": 1,
        "method": "Runtime.enable"
    }"#
        .into(),
    );
    write.send(enable_runtime).await?;

    let (keep_alive_tx, keep_alive_rx) = futures::channel::mpsc::unbounded();
    tokio::spawn(keep_alive(keep_alive_tx));
    let keep_alive_to_ws = keep_alive_rx.map(Ok).forward(write);

    let print_ws_messages = {
        read.for_each(|message| {
            async {
                let message = message.unwrap().into_text().unwrap();
                log::info!("{}", message);
                let message: Result<DevtoolsEvent, serde_json::Error> =
                    serde_json::from_str(&message);
                match message {
                    Ok(message) => match message {
                        DevtoolsEvent::ConsoleAPICalled(event) => match event.log_type.as_str() {
                            "log" => println!("{}", style(event).blue()),
                            "error" => println!("{}", style(event).red()),
                            _ => println!("unknown console event: {}", event),
                        },
                        DevtoolsEvent::ExceptionThrown(event) => {
                            println!("{}", style(event).bold().red())
                        }
                    },
                    Err(e) => {
                        // this event was not parsed as a DevtoolsEvent
                        // TODO: change this to a warn after chrome-devtools-rs is parsing all messages
                        log::info!("this event was not parsed as a DevtoolsEvent:\n{}", e);
                    }
                };
            }
        })
    };
    pin_mut!(keep_alive_to_ws, print_ws_messages);
    future::select(keep_alive_to_ws, print_ws_messages).await;
    Ok(())
}

async fn keep_alive(tx: futures::channel::mpsc::UnboundedSender<Message>) {
    let duration = Duration::from_millis(1000 * KEEP_ALIVE_INTERVAL);
    let mut interval = time::interval(duration);

    let mut id = 2;

    loop {
        interval.tick().await;
        let keep_alive_message = format!(
            r#"{{
              "id": {},
              "method": "Runtime.getIsolateId"
            }}"#,
            id
        );

        tx.unbounded_send(Message::Text(keep_alive_message.into()))
            .unwrap();
        id += 1;
    }
}
