use crate::http;
use crate::settings::global_user::GlobalUser;
use crate::settings::toml::Target;

use crate::terminal::{colored_json_string, styles};
/// `wrangler tail` allows Workers users to collect logs from their deployed Workers.
/// When a user runs `wrangler tail`, several things happen:
///     1. wrangler asks the Cloudflare API to send log events to a Durable Object, which will act as a log forwarder.
///        The API returns the address of the log forwarder.
///     3. wrangler connects to the log forwarder with a Websocket.
///     5. Upon receipt, wrangler prints log events to STDOUT.
use anyhow::Result;
use cloudflare::endpoints::workers::{CreateTail, CreateTailParams};
use cloudflare::framework::apiclient::ApiClient;
use futures::stream::StreamExt;
use serde::{Deserialize, Serialize};
use std::convert::TryInto;

fn print_logs_pretty(msg: String) -> Result<()> {
    let parsed = serde_json::from_str::<LogResponse>(&msg).unwrap();

    let secs = (parsed.event_timestamp / 1000).try_into().unwrap();

    let timestamp = chrono::NaiveDateTime::from_timestamp(secs, 0);

    println!(
        "{} {} --> {} @ {} UTC",
        parsed.event.request.method,
        styles::url(parsed.event.request.url),
        parsed.outcome.to_uppercase(),
        timestamp.time()
    );

    if !parsed.exceptions.is_empty() {
        println!("  Exceptions:");
        parsed.exceptions.iter().for_each(|exception| {
            println!(
                "{}",
                styles::warning(format!("{}: {}", exception.name, exception.message))
            );
        });
    }

    if !parsed.logs.is_empty() {
        println!("  Logs:");
        parsed.logs.iter().for_each(|log| {
            let message = colored_json_string(&log.message);
            let messages = if let Ok(m) = message {
                m
            } else {
                "Error: Failed to convert encoded message to string".to_string()
            };

            let output = match log.level.as_str() {
                "assert" | "error" => format!("{}", styles::warning(messages)),
                "warn" => format!("{}", styles::highlight(messages)),
                "trace" | "debug" => {
                    format!("{}", styles::cyan(messages))
                }
                _ => format!("{}", styles::bold(messages)),
            };

            println!("\t{}", output);
        });
    }
    Ok(())
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct LogResponse {
    outcome: String,
    script_name: Option<String>,
    exceptions: Vec<LogException>,
    logs: Vec<LogMessage>,
    event_timestamp: usize,
    event: RequestEvent,
}

#[derive(Debug, Serialize, Deserialize)]
struct LogException {
    name: String,
    message: String,
    timestamp: usize,
}

#[derive(Debug, Serialize, Deserialize)]
struct LogMessage {
    message: serde_json::Value,
    level: String,
    timestamp: usize,
}

#[derive(Debug, Serialize, Deserialize)]
struct RequestEvent {
    request: RequestEventData,
}

#[derive(Debug, Serialize, Deserialize)]
struct RequestEventData {
    url: String,
    method: String,
}

// Main loop, listen for websocket messages or interrupt
async fn listen_tail(format: String, tail_tag: String) -> Result<(), ()> {
    // ws listener setup
    let listen_tail_endpoint = format!("wss://tail.developers.workers.dev/{}/ws", tail_tag);
    let cant_connect_msg = format!(
        "Can't connect to the log forwarding surver at {}",
        listen_tail_endpoint
    );
    let (mut socket, _) =
        tokio_tungstenite::connect_async(url::Url::parse(&listen_tail_endpoint).unwrap())
            .await
            .expect(&cant_connect_msg);
    log::debug!(
        "Connected to the log forwarding server at {}",
        listen_tail_endpoint
    );

    loop {
        tokio::select! {
            maybe_incoming = socket.next()  => {
                if let Some(Ok(msg)) = maybe_incoming {
                    if format == "pretty" {
                        print_logs_pretty(msg.to_string()).unwrap();
                    } else {
                        println!("{}", msg);
                    }
                }
            },
            _ = tokio::signal::ctrl_c() => {
                socket.close(None).await.expect("Failed to close socket after ctrlc");
                break Ok(());
            },
        }
    }
}

pub fn start(format: String, target: Target, user: GlobalUser) -> Result<()> {
    // Tell API to start tail
    let client = http::cf_v4_client(&user)?;

    let res = client.request(&CreateTail {
        account_identifier: target.account_id.load()?,
        script_name: &target.name,
        params: CreateTailParams { url: None },
    });

    match res {
        Ok(resp) => {
            let start_tail_future = async move { listen_tail(format, resp.result.id).await };

            match tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()
                .unwrap()
                .block_on(start_tail_future)
            {
                Ok(_) => Ok(()),
                Err(e) => anyhow::bail!("Websocket listening failed with err {:?}", e),
            }
        }
        Err(e) => anyhow::bail!("POST tail failed with err {:?}", e),
    }
}
