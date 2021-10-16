use std::convert::Infallible;
use std::fmt::Display;
use std::net::SocketAddr;
use std::sync::mpsc::Sender;
use std::sync::Arc;
use std::time::Duration;

use chrome_devtools as protocol;

use futures_util::future::TryFutureExt;
use futures_util::sink::SinkExt;
use futures_util::stream::{SplitStream, StreamExt};
use http::header::SEC_WEBSOCKET_KEY;
use http::{HeaderValue, Request, Response, StatusCode};
use hyper::upgrade::Upgraded;
use hyper::{Body, Server};
use log::{debug, error, info, trace, warn};
use tokio::try_join;
use tokio_stream::wrappers::UnboundedReceiverStream;
use tokio_tungstenite::tungstenite::error::ProtocolError;

use crate::terminal::colored_json_string;
use crate::terminal::message::{Message, StdErr, StdOut};
use protocol::domain::runtime::event::Event::ExceptionThrown;
use tokio::net::TcpStream;
use tokio::sync::mpsc;
use tokio::time::sleep;

use tokio_tungstenite::{connect_async, tungstenite, MaybeTlsStream, WebSocketStream};

use anyhow::{anyhow, Result};
use url::Url;

use super::ServerConfig;

const KEEP_ALIVE_INTERVAL: u64 = 10;
const DEVTOOLS_PORT: u16 = 9230;

/// connect to a Workers runtime WebSocket emitting the Chrome Devtools Protocol
/// parse all console messages, and print them to stdout
///
/// `inspect` should be the name of the worker if `--inspect` is passed, or `None` otherwise.
pub async fn listen(
    socket_url: Url,
    server_config: ServerConfig,
    inspect: Option<String>,
    refresh_session_sender: Option<Sender<Option<()>>>,
) -> Result<()> {
    // we loop here so we can issue a reconnect when something
    // goes wrong with the websocket connection
    loop {
        let sender = refresh_session_sender.clone();
        let mut ws_stream = connect_retry(&socket_url, sender).await;

        // console.log messages are in the Runtime domain
        // we must signal that we want to receive messages from the Runtime domain
        // before they will be sent
        let enable_runtime = protocol::runtime::SendMethod::Enable(1.into());
        let enable_runtime = serde_json::to_string(&enable_runtime)?;
        let enable_runtime = tungstenite::protocol::Message::Text(enable_runtime);
        ws_stream.send(enable_runtime).await?;

        // parse all incoming messages and print them to stdout
        if let Some(worker_name) = &inspect {
            StdErr::help(&format!(
                "Open chrome://inspect, click 'Configure', and add localhost:{}",
                DEVTOOLS_PORT
            ));

            // Construct our SocketAddr to listen on...
            let addr = SocketAddr::from(([127, 0, 0, 1], DEVTOOLS_PORT));

            // And a MakeService to handle each connection...
            use hyper::service::{make_service_fn, service_fn};
            use rand::Rng;
            let random_bytes = rand::thread_rng().gen();
            let uuid = uuid::Builder::from_bytes(random_bytes)
                .set_variant(uuid::Variant::RFC4122)
                .set_version(uuid::Version::Random)
                .build();

            let remote_stream = Arc::new(tokio::sync::Mutex::new(ws_stream));
            let make_service = make_service_fn(|_conn| {
                let socket_url = socket_url.clone();
                let remote_stream = remote_stream.clone();
                let listening_address = server_config.listening_address.to_string();
                let worker_name = worker_name.clone();
                async move {
                    Ok::<_, Infallible>(service_fn(move |req| {
                        devtools_http_request(
                            req,
                            socket_url.clone(),
                            listening_address.clone(),
                            uuid,
                            remote_stream.clone(),
                            worker_name.clone(),
                        )
                    }))
                }
            });

            // Then bind and serve indefinitely.
            let server = Server::bind(&addr).serve(make_service);
            if server.await.is_ok() {
                error!("connection closed!!");
                break Ok(());
            } else {
                info!("restarting HTTP server");
            }
        } else {
            let (write, read) = ws_stream.split();

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
}

struct WebsocketError {
    from_chrome: bool,
    inner: tungstenite::Error,
}

impl Display for WebsocketError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let direction = if self.from_chrome {
            "chrome -> edgeworker"
        } else {
            "edgeworker -> chrome"
        };
        write!(f, "{}: {}", direction, self.inner)
    }
}

async fn websocket_handle_events(
    local_stream: WebSocketStream<Upgraded>,
    remote_stream: &mut WebSocketStream<MaybeTlsStream<TcpStream>>,
) -> std::result::Result<(), WebsocketError> {
    info!("opened websocket stream between Chrome and edgeworker");
    // proxy to the upstream websocket
    let (local_write, local_read) = local_stream.split();
    let (remote_write, remote_read) = remote_stream.split();
    let edgeworker_to_chrome = remote_read
        .inspect(|message| match message {
            Ok(m) => debug!("edgeworker -> chrome: {}", m),
            Err(e) => warn!("{}", e),
        })
        .forward(local_write)
        .map_err(|e| WebsocketError {
            inner: e,
            from_chrome: false,
        });
    let chrome_to_edgeworker = local_read
        .filter(|message| {
            let should_forward = match message {
                Ok(m) => {
                    debug!("chrome -> edgeworker: {}", m);
                    true
                }
                Err(e) => {
                    warn!("{}", e);
                    // When the user closes the inspect window, chrome will abruptly terminate the
                    // TCP connection, causing tungstenite to mark the stream as poisoned.
                    // However, this isn't actually a fatal error: if the user opens a new window,
                    // Chrome will continue to send packets through the websocket.
                    // Hide this error from the remote so it doesn't give an error when new packets come it.
                    !matches!(e, tungstenite::Error::ConnectionClosed)
                }
            };
            std::future::ready(should_forward)
        })
        .forward(remote_write)
        .map_err(|e| WebsocketError {
            inner: e,
            from_chrome: true,
        });
    try_join!(edgeworker_to_chrome, chrome_to_edgeworker)?;
    Ok(())
}

async fn start_websocket(
    req: Request<Body>,
    remote_stream: Arc<tokio::sync::Mutex<WebSocketStream<MaybeTlsStream<TcpStream>>>>,
) -> Result<Response<Body>> {
    let key = req
        .headers()
        .get(SEC_WEBSOCKET_KEY)
        .expect("missing SEC_WEBSOCKET_KEY header")
        .clone();

    tokio::spawn(async move {
        let upgraded = hyper::upgrade::on(req).await.expect("failed to upgrade");
        let local_stream = WebSocketStream::from_raw_socket(
            upgraded,
            tokio_tungstenite::tungstenite::protocol::Role::Server,
            None,
        )
        .await;
        // NOTE: keeps the mutex locked until the connection is closed.
        // This is fine because we don't support simultaneous clients anyway.
        let mut remote_stream = remote_stream.lock().await;
        use tungstenite::Error;
        match websocket_handle_events(local_stream, &mut remote_stream).await {
            Ok(()) => {}
            Err(WebsocketError {
                inner:
                    Error::ConnectionClosed
                    | Error::Protocol(ProtocolError::ResetWithoutClosingHandshake),
                ..
            }) => {
                info!("websocket connection closed");
            }
            Err(e) => panic!("failed to run websocket server: {}", e),
        }
    });

    use hyper::header::{CONNECTION, SEC_WEBSOCKET_ACCEPT, UPGRADE};

    let mut upgrade_response = Response::builder()
        .status(StatusCode::SWITCHING_PROTOCOLS)
        .body(Body::empty())
        .unwrap();

    let headers = upgrade_response.headers_mut();
    headers.insert(UPGRADE, HeaderValue::from_static("WebSocket"));
    headers.insert(CONNECTION, HeaderValue::from_static("Upgrade"));
    headers.insert(SEC_WEBSOCKET_ACCEPT, key);

    Ok(upgrade_response)
}

async fn devtools_http_request(
    req: Request<Body>,
    remote_ws: Url,
    listening_address: String,
    uuid: uuid::Uuid,
    remote_stream: Arc<tokio::sync::Mutex<WebSocketStream<MaybeTlsStream<TcpStream>>>>,
    worker_name: String,
) -> Result<Response<Body>> {
    let path = req.uri().path();
    if path == "/json/version" {
        // TODO: get actual protocol version from remote
        let version = format!(
            r#"{{
            "Browser": "wrangler/v{version}", "Protocol-Version": "1.3",
        }}"#,
            version = env!("CARGO_PKG_VERSION")
        );
        return Response::builder().body(version.into()).map_err(Into::into);
    } else if path == "/json" || path == "/json/list" {
        let devtools_info = format!(
            r#"
        [ {{
            "description": "wrangler dev --inspect instance",
            "devtoolsFrontendUrl": "devtools://devtools/bundled/js_app.html?experiments=true&v8only=true&ws=localhost:{port}{path}",
            "devtoolsFrontendUrlCompat": "devtools://devtools/bundled/inspector.html?experiments=true&v8only=true&ws=localhost:{port}{path}",
            "id": "{uuid}",
            "type": "node",
            "title": "wrangler[{worker}]",
            "url": "http://{local_address}",
            "faviconUrl": "https://workers.cloudflare.com/resources/logo/logo.svg",
            "webSocketDebuggerUrl": "ws://localhost:{port}{path}"
          }} ]
        "#,
            uuid = uuid,
            worker = worker_name,
            local_address = listening_address,
            port = DEVTOOLS_PORT,
            path = remote_ws.path()
        );

        trace!(
            "sending json description for {} back:{}",
            remote_ws.as_str(),
            devtools_info
        );
        return Response::builder()
            .header("Content-Type", "application/json")
            .body(devtools_info.into())
            .map_err(Into::into);
    } else if path == remote_ws.path() {
        return start_websocket(req, remote_stream).await;
    }
    warn!("inspect: unknown request URL {}: {:?}", req.uri(), req);
    Response::builder()
        .status(StatusCode::NOT_FOUND)
        .body("".into())
        .map_err(Into::into)
}

// Endlessly retry connecting to the chrome devtools instance with exponential backoff.
// The backoff maxes out at 60 seconds.
async fn connect_retry(
    socket_url: &Url,
    sender: Option<Sender<Option<()>>>,
) -> WebSocketStream<MaybeTlsStream<TcpStream>> {
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
                log::info!(
                    "Failed to connect to devtools instance: {}. Retrying in {} seconds",
                    e,
                    wait_seconds
                );
                sleep(Duration::from_secs(wait_seconds)).await;
                wait_seconds *= 2;
                if let (Some(sender), tungstenite::Error::Http(resp)) = (&sender, e) {
                    if resp.status().as_u16() >= 400 && resp.status().as_u16() < 500 {
                        sender.send(Some(())).ok();
                    }
                }
                if wait_seconds > maximum_wait_seconds {
                    // max out at 60 seconds
                    wait_seconds = maximum_wait_seconds;
                }
                log::info!("Attempting to reconnect to devtools instance...");
            }
        }
    }
}

fn print_json(value: Result<serde_json::Value, serde_json::Error>, fallback: String) {
    if let Ok(json) = value {
        if let Ok(json_str) = colored_json_string(&json) {
            println!("{}", json_str);
        } else {
            StdOut::message(fallback.as_str());
        }
    } else {
        println!("{}", fallback);
    }
}

async fn print_ws_messages(
    mut read: SplitStream<WebSocketStream<MaybeTlsStream<TcpStream>>>,
) -> Result<()> {
    while let Some(message) = read.next().await {
        let message = message?;
        let message_text = message.to_text().unwrap();
        log::info!("{}", message_text);

        let parsed_message: Result<protocol::Runtime> = serde_json::from_str(message_text)
            .map_err(|e| anyhow!("Failed to parse event:\n{}", e));

        match parsed_message {
            Ok(protocol::Runtime::Event(ExceptionThrown(params))) => {
                let default_description = "N/A".to_string();
                let description = params
                    .exception_details
                    .exception
                    .description
                    .as_ref()
                    .unwrap_or(&default_description);

                StdOut::message(&format!(
                    "{} at line {:?}, col {:?}",
                    description,
                    params.exception_details.line_number,
                    params.exception_details.column_number,
                ));

                let json_parse = serde_json::to_value(params.clone());
                print_json(json_parse, format!("{:?}", params));
            }
            Ok(protocol::Runtime::Event(event)) => {
                // Try to parse json to pretty print, otherwise just print string
                let json_parse: Result<serde_json::Value, serde_json::Error> =
                    serde_json::from_str(&*event.to_string());
                print_json(json_parse, event.to_string());
            }
            Ok(protocol::Runtime::Method(_)) => {}
            Err(err) => log::debug!("{}", err),
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
        if let Err(e) = tx.send(keep_alive_message) {
            log::error!("failed to send keepalive message: {}", e);
        }
        id += 1;
        delay = sleep(duration);
    }
}
