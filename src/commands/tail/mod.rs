pub mod api;
/// `wrangler tail` allows Workers users to collect logs from their deployed Workers.
/// When a user runs `wrangler tail`, several things happen:
///     1. wrangler asks the Cloudflare API to send log events to a Durable Object,
///        which will act as a log forwarder. The API returns the URL of the log forwarder.
///     2. wrangler connects to the log forwarder using a WebSocket.
///     3. Upon receipt of messages, wrangler prints log events to stdout.
pub mod event;
pub mod filter;
pub mod websocket;

use crate::settings::global_user::GlobalUser;
use crate::terminal::styles;

use api::Tail;
use websocket::{TailOptions, WebSocketTail};

use anyhow::Result;
use indicatif::{ProgressBar, ProgressStyle};
use url::Url;

/// Runs a tail session from creation to deletion.
///
/// It can be interrupted by:
/// * an API error when creating the tail
/// * a WebSocket error when receiving events
/// * a user typing ctrl-c
/// * an expiration of the tail
///
/// A fancy progress bar is also updated throughout the session.
pub async fn run(
    user: GlobalUser,
    account_id: String,
    script_name: String,
    url: Option<Url>,
    options: TailOptions,
) -> Result<()> {
    let progress = &mut ProgressBar::new_spinner()
        .with_style(ProgressStyle::default_spinner().template("{spinner} {msg}"));
    progress.enable_steady_tick(20);
    progress.set_message("Creating tail...");

    let tail = &mut Tail::new(user, account_id, script_name, url);
    tail.create().await?;

    if tail.is_web_socket() {
        progress.set_message("Connecting to tail...");

        match &mut WebSocketTail::connect(tail.clone(), options).await {
            Ok(websocket) => {
                progress.abandon_with_message(&format!(
                    "Connected! Streaming logs from {}... (ctrl-c to quit)",
                    styles::bold(&tail.script_name)
                ));

                if let Err(err) = websocket.update().await {
                    log::warn!("{}", err);
                };
                if let Err(err) = websocket.read().await {
                    log::warn!("{}", err);
                }
            }
            Err(err) => progress.abandon_with_message(&format!("{}", err)),
        }
    } else {
        progress.set_message(&format!(
            "Forwarding logs from {} to {} (ctrl-c to quit)",
            styles::bold(&tail.script_name),
            styles::url(
                tail.url
                    .clone()
                    .map(String::from)
                    .unwrap_or_else(|| "an endpoint".to_owned())
            )
        ));

        if let Err(err) = loop {
            tokio::select! {
                _ = tokio::signal::ctrl_c() => break Ok(()),
                _ = tokio::time::sleep_until(tail.expires_at) => if let Err(err) = tail.keep_alive().await { break Err(err) }
            }
        } {
            progress.abandon_with_message(&format!("{}", err));
        }
    }

    tail.delete().await?;
    Ok(())
}
