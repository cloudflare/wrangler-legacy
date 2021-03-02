/// `wrangler tail` allows Workers users to collect logs from their deployed Workers.
/// When a user runs `wrangler tail`, several things happen:
///     1. Wrangler initiates a tail Session by making a request to the Workers API /tail endpoint,
///        providing an unguessable URL to an HTTP server.
///     2. The Workers API binds the URL to a [Trace Worker], and directs all `console` and
///        exception logging to the Trace Worker, which POSTs each batch of logs as a JSON
///        payload to the provided URL.
///     3. Wrangler establishes a WebSocket connection with the URL, which yields a stream
///        of logs which are sent to STDOUT.
mod session;
mod shutdown;

use session::Session;
use shutdown::ShutdownHandler;

use futures_util::StreamExt;
use tokio::runtime::Runtime as TokioRuntime;
use tokio_tungstenite::connect_async;
use tokio_tungstenite::tungstenite::protocol::Message;
use url::Url;
use uuid::Uuid;

use crate::settings::global_user::GlobalUser;
use crate::settings::toml::Target;
use crate::terminal::emoji;

pub struct Tail;

impl Tail {
    pub fn run(target: Target, user: GlobalUser) -> Result<(), failure::Error> {
        print_startup_message(&target.name);

        let mut runtime = TokioRuntime::new()?;

        runtime.block_on(async {
            // Create three [one-shot](https://docs.rs/tokio/0.2.16/tokio/sync#oneshot-channel)
            // channels for handling ctrl-c. Each channel has two parts:
            // tx: Transmitter
            // rx: Receiver
            let (tx, rx) = tokio::sync::oneshot::channel(); // shutdown short circuit
            let mut shutdown_handler = ShutdownHandler::new();
            let session_rx = shutdown_handler.subscribe();

            let listener = tokio::spawn(shutdown_handler.run(rx));

            let (tail_url, ws_url) = generate_tail_urls();
            let ws_session = tokio::spawn(listen_to_websocket(ws_url));

            let session = tokio::spawn(Session::run(target, user, tail_url, session_rx, tx));

            // TODO(later): first, register tail with the API, /then/ connect to the WebSocket.
            let res = tokio::try_join!(
                async { listener.await? },
                async { session.await? },
                // FIXME(now): how to pass ctrl-c to disconnect the WebSocket connection??
                async { ws_session.await? },
            );

            match res {
                Ok(_) => Ok(()),
                Err(e) => Err(e),
            }
        })
    }
}

fn print_startup_message(worker_name: &str) {
    // Note that we use eprintln!() throughout this module; this is because we want any
    // helpful output to not be mixed with actual log JSON output, so we use this macro
    // to print messages to stderr instead of stdout (where log output is printed).
    eprintln!(
        "{} Streaming logs from the Worker script \"{}\".",
        emoji::TAIL,
        worker_name,
    );
}

// TODO(now): will change to a Cloudflare-owned zone.
const TAIL_HOSTNAME: &str = "gmad.dev";

// TODO(now): subject to change.
fn generate_tail_urls() -> (String, String) {
    let uuid = Uuid::new_v4();
    let hostname = format!("{}.{}", uuid, TAIL_HOSTNAME);
    let tail_url = format!("https://{}/channel", hostname);
    let ws_url = format!("wss://{}/ws", hostname);
    (tail_url, ws_url)
}

/// Connects to a WebSocket and prints text frames to stdout.
async fn listen_to_websocket(url: String) -> Result<(), failure::Error> {
    let parsed_url = Url::parse(&url).unwrap();
    let (stream, _) = connect_async(parsed_url).await.unwrap();
    let (_, read) = stream.split();
    read.for_each(|frame| async {
        match frame.unwrap() {
            Message::Text(message) => println!("{}", message),
            // Silently ignore other message types, in case we want to
            // introduce more functionality in the future.
            _ => {}
        }
    })
    .await;
    Ok(())
}
