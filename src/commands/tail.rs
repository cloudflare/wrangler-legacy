use crate::settings::global_user::GlobalUser;
use crate::settings::toml::Target;

/// `wrangler tail` allows Workers users to collect logs from their deployed Workers.
/// When a user runs `wrangler tail`, several things happen:
///     1. wrangler asks the Cloudflare API to send log events to a Durable Object, which will act as a log forwarder.
///        The API returns the address of the log forwarder.
///     3. wrangler connects to the log forwarder with a Websocket.
///     5. Upon receipt, wrangler prints log events to STDOUT.
use anyhow::{anyhow, Result};
use cloudflare::framework::response;
use futures::stream::StreamExt;

#[derive(serde::Deserialize, Debug)]
struct TailResult {
    id: String,
    expires_at: String,
}

async fn get_tail_tag(resp: reqwest::Response) -> Result<String, reqwest::Error> {
    let body: response::ApiSuccess<TailResult> = resp.json().await?;
    Ok(body.result.id)
}

// Main loop, listen for websocket messages or interrupt
async fn listen_tail(tail_tag: String) -> Result<(), ()> {
    // ws listener setup
    let listen_tail_endpoint = format!("wss://tail.developers.workers.dev/{}/ws", tail_tag);
    let (mut socket, _) =
        tokio_tungstenite::connect_async(url::Url::parse(&listen_tail_endpoint).unwrap())
            .await
            .expect("Can't connect");
    eprintln!("Connected to the log forwarding server at {}", listen_tail_endpoint);
 
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

pub async fn start(target: Target, user: GlobalUser) -> anyhow::Result<()> {
    // Tell API to start tail. For now, do client logic here. TODO add endpoint to cloudflare-rs
    let start_tail_endpoint = format!(
        "https://api.cloudflare.com/client/v4/accounts/{}/workers/scripts/{}/tails",
        target.account_id, target.name
    );
    let client = reqwest::Client::new().post(start_tail_endpoint);
    let req_builder: reqwest::RequestBuilder = match user {
        GlobalUser::GlobalKeyAuth { email, api_key } => {
            let mut headers = reqwest::header::HeaderMap::new();
            let email_hdr = reqwest::header::HeaderName::from_static("x-auth-email");
            let api_key_hdr = reqwest::header::HeaderName::from_static("x-auth-key");
            headers.insert(email_hdr, email.parse().unwrap());
            headers.insert(api_key_hdr, api_key.parse().unwrap());
            client.headers(headers)
        }
        GlobalUser::TokenAuth { api_token } => client.bearer_auth(api_token),
    };
    let res = req_builder.send().await;

    match res {
        Ok(resp) => match get_tail_tag(resp).await {
            Err(e) => Err(anyhow!("Getting body failed: {:?}", e)),
            Ok(tag) => match listen_tail(tag).await {
                Err(e) => Err(anyhow!("Websocket err: {:?}", e)),
                _ => Ok(()),
            },
        },
        Err(e) => anyhow::bail!("POST tail failed with err {:?}", e),
    }
}
