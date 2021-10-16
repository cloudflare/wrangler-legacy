use crate::http::feature::user_agent;

use super::api::Tail;
use super::event::{TraceEvent, PROTOCOL_ID};
use super::filter::TraceFilter;

use anyhow::Result;
use futures_util::{SinkExt, StreamExt};
use serde::{Deserialize, Serialize};
use std::borrow::Cow;
use std::str::FromStr;
use tokio::net::TcpStream;
use tokio_tungstenite::tungstenite::error::Error::{AlreadyClosed, ConnectionClosed};
use tokio_tungstenite::tungstenite::handshake::client::Request;
use tokio_tungstenite::tungstenite::protocol::frame::coding::CloseCode;
use tokio_tungstenite::tungstenite::protocol::{CloseFrame, Message};
use tokio_tungstenite::{MaybeTlsStream, WebSocketStream};

/// The format to print a `TraceEvent`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TailFormat {
    Json,
    Pretty,
}

impl FromStr for TailFormat {
    type Err = anyhow::Error;
    fn from_str(string: &str) -> Result<Self> {
        match string {
            "pretty" => Ok(TailFormat::Pretty),
            "json" => Ok(TailFormat::Json),
            _ => Ok(TailFormat::Json),
        }
    }
}

/// Options that are sent to the `WebSocketTail`.
#[derive(Serialize)]
pub struct TailOptions {
    #[serde(skip_serializing)]
    pub once: bool,
    #[serde(skip_serializing)]
    pub format: TailFormat,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub filters: Vec<Box<dyn TraceFilter>>,
}

/// A tail that sends `TraceEvent`s to a WebSocket.
pub struct WebSocketTail {
    pub tail: Tail,
    pub options: TailOptions,
    pub websocket: WebSocketStream<MaybeTlsStream<TcpStream>>,
    pub closed: bool,
}

impl WebSocketTail {
    /// Connects to WebSocket tail.
    pub async fn connect(tail: Tail, options: TailOptions) -> Result<Self> {
        if tail.id.is_none() && tail.url.is_none() && !tail.is_web_socket() {
            anyhow::bail!("Precondition failed for WebSocket tail: {:?}", &tail);
        }
        let request = Request::builder()
            .uri(&tail.url.clone().map(String::from).unwrap())
            .header("User-Agent", user_agent())
            .header("Sec-WebSocket-Protocol", PROTOCOL_ID)
            .body(())?;
        log::info!("Connecting to WebSocket tail: {:?}", request);
        match tokio_tungstenite::connect_async(request).await {
            Ok((websocket, _)) => Ok(Self {
                tail,
                options,
                websocket,
                closed: false,
            }),
            Err(err) => anyhow::bail!("Failed to create WebSocket tail: {}", err),
        }
    }

    /// Reads a message from the WebSocket and prints it.
    pub async fn read_once(&mut self) -> Result<()> {
        tokio::select! {
            frame = self.websocket.next() => {
                match frame {
                    Some(Ok(message)) if message.is_text() || message.is_binary() => {
                        match self.options.format {
                            TailFormat::Json => {
                                println!("{}", message);
                                Ok(())
                            },
                            TailFormat::Pretty => match serde_json::from_str::<TraceEvent>(&message.to_string()) {
                                Ok(event) => {
                                    println!("{}", event);
                                    Ok(())
                                },
                                Err(err) => {
                                    log::debug!("Failed to pretty-print tail: {}", err);
                                    self.close(CloseCode::Protocol, "wrangler is closing due to a protocol violation").await
                                },
                            }
                        }
                    },
                    Some(Ok(message)) if message.is_close() => {
                        anyhow::bail!("Received close from WebSocket tail: {}", message)
                    },
                    Some(Err(err)) => {
                        log::debug!("Received error from WebSocket tail: {}", err);
                        self.close(CloseCode::Abnormal, "wrangler is closing due to an error").await
                    },
                    _ => Ok(()),
                }
            },
            _ = tokio::signal::ctrl_c() => {
                self.close(CloseCode::Away, "wrangler is closing due to ctrl-c").await
            },
            _ = tokio::time::sleep_until(self.tail.expires_at) => {
                self.close(CloseCode::Normal, "wrangler is closing due to expiration").await
            }
        }
    }

    /// Reads and prints messages from the WebSocket in a loop.
    pub async fn read(&mut self) -> Result<()> {
        loop {
            if self.closed {
                break Ok(());
            }
            match self.read_once().await {
                Err(err) => break Err(err),
                Ok(_) if self.options.once => {
                    break self
                        .close(
                            CloseCode::Normal,
                            "wrangler is closing after receiving first log",
                        )
                        .await
                }
                _ => {}
            };
        }
    }

    /// Writes a text message to the WebSocket.
    pub async fn write(&mut self, message: String) -> Result<()> {
        log::debug!("Sending message to WebSocket tail: {}", message);
        match self.websocket.send(Message::Text(message)).await {
            Err(err) => anyhow::bail!("Failed to write to WebSocket tail: {}", err),
            _ => Ok(()),
        }
    }

    /// Sends the tail filters to the WebSocket.
    pub async fn update(&mut self) -> Result<()> {
        match self.options.filters.is_empty() {
            false => match serde_json::to_string(&self.options) {
                Ok(options) => self.write(options).await,
                Err(err) => anyhow::bail!("Failed to deserialize options: {}", err),
            },
            true => Ok(()),
        }
    }

    /// Closes the WebSocket.
    pub async fn close(&mut self, code: CloseCode, reason: &str) -> Result<()> {
        if self.closed {
            return Ok(());
        } else {
            self.closed = true;
        }
        let frame = CloseFrame {
            code,
            reason: Cow::Borrowed(reason),
        };
        match self.websocket.close(Some(frame)).await {
            Ok(_) => {
                log::info!("Closed WebSocket tail: {}", reason);
                Ok(())
            }
            Err(AlreadyClosed | ConnectionClosed) => Ok(()),
            Err(err) => anyhow::bail!("Failed to close WebSocket tail: {}", err),
        }
    }
}
