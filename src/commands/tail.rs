use crate::http;
use crate::settings::global_user::GlobalUser;
use crate::settings::toml::Target;

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

#[derive(serde::Deserialize, Debug)]
struct TailResult {
    id: String,
    expires_at: String,
}

// Main loop, listen for websocket messages or interrupt
async fn listen_tail(tail_tag: String) -> Result<(), ()> {
    // ws listener setup
    let listen_tail_endpoint = format!("wss://tail.developers.workers.dev/{}/ws", tail_tag);
    let (mut socket, _) =
        tokio_tungstenite::connect_async(url::Url::parse(&listen_tail_endpoint).unwrap())
            .await
            .expect("Can't connect");
    log::debug!(
        "Connected to the log forwarding server at {}",
        listen_tail_endpoint
    );

    loop {
        tokio::select! {
            maybe_incoming = socket.next()  => {
                if let Some(Ok(msg)) = maybe_incoming {
                    println!("{}", msg);
                }
            },
            _ = tokio::signal::ctrl_c() => {
                socket.close(None).await.expect("Failed to close socket after ctrlc");
                break Ok(());
            },
        }
    }
}

pub fn start(target: Target, user: GlobalUser) -> Result<()> {
    // Tell API to start tail
    let client = http::cf_v4_client(&user)?;

    let res = client.request(&CreateTail {
        account_identifier: &target.account_id,
        script_name: &target.name,
        params: CreateTailParams { url: None },
    });

    match res {
        Ok(resp) => {
            let start_tail_future = async move { listen_tail(resp.result.id).await };

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
