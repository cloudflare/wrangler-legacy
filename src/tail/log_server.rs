use crate::terminal::{colored_json_string, emoji, styles};
use anyhow::Result;
use hyper::server::conn::AddrIncoming;
use hyper::server::Builder;
use hyper::service::{make_service_fn, service_fn};
use hyper::{Body, Method, Request, Response, Server, StatusCode};
use serde::{Deserialize, Serialize};
use std::convert::TryInto;
use tokio::sync::oneshot::Receiver;

pub struct LogServer {
    server: Builder<AddrIncoming>,
    shutdown_rx: Receiver<()>,
    format: String,
}

/// LogServer is just a basic HTTP server running locally; it listens for POST requests on the root
/// path and simply prints the JSON body of each request as its own line to STDOUT.
impl LogServer {
    pub fn new(port: u16, shutdown_rx: Receiver<()>, format: String) -> LogServer {
        // Start HTTP echo server that prints whatever is posted to it.
        let addr = ([127, 0, 0, 1], port).into();

        let server = Server::bind(&addr);

        LogServer {
            server,
            shutdown_rx,
            format,
        }
    }

    pub async fn run(self) -> Result<()> {
        let format = self.format.clone(); // Having to clone this so much is a little gross TODO

        let service = make_service_fn(move |_| {
            let format = format.clone();
            async move {
                let format = format.clone();
                Ok::<_, hyper::Error>(service_fn(move |req| {
                    let format = format.clone();
                    async move {
                        let format = format.clone();
                        match format.as_str() {
                            "pretty" => print_logs_pretty(req).await,
                            "json" => print_logs_json(req).await,
                            _ => unreachable!(),
                        }
                    }
                }))
            }
        });

        let server = self.server.serve(service);

        // The shutdown receiver listens for a one shot message from our sigint handler as a signal
        // to gracefully shut down the hyper server.
        let shutdown_rx = self.shutdown_rx;

        let graceful = server.with_graceful_shutdown(async {
            shutdown_rx.await.ok();
        });

        graceful.await?;

        Ok(())
    }
}

async fn print_logs_json(req: Request<Body>) -> Result<Response<Body>> {
    match (req.method(), req.uri().path()) {
        (&Method::POST, "/") => {
            let whole_body = hyper::body::to_bytes(req.into_body()).await?;
            println!(
                "{}",
                std::str::from_utf8(&whole_body).expect("failed to deserialize tail log body")
            );

            Ok(Response::new(Body::from("Success")))
        }
        _ => {
            let mut bad_request = Response::default();
            *bad_request.status_mut() = StatusCode::BAD_REQUEST;
            Ok(bad_request)
        }
    }
}

async fn print_logs_pretty(req: Request<Body>) -> Result<Response<Body>> {
    match (req.method(), req.uri().path()) {
        (&Method::POST, "/") => {
            let whole_body = hyper::body::to_bytes(req.into_body()).await?;

            let parsed = serde_json::from_slice::<LogResponse>(&whole_body).map_err(|e| {
                println!("{}", styles::warning("Error parsing response body!"));
                println!(
                    "This is not a problem with your worker, it's a problem with Wrangler.\nPlease file an issue on our GitHub page, with a minimal reproducible example of\nthe script that caused this error and a description of what happened."
                );
                e
            })?;

            let secs = (parsed.event_timestamp / 1000).try_into().unwrap();

            let timestamp = chrono::NaiveDateTime::from_timestamp(secs, 0);

            println!(
                "{}{} {} --> {} @ {} UTC",
                emoji::EYES,
                parsed.event.request.method,
                styles::url(parsed.event.request.url),
                parsed.outcome.to_uppercase(),
                timestamp.time()
            );

            if !parsed.exceptions.is_empty() {
                println!("  Exceptions:");
                parsed.exceptions.iter().for_each(|exception| {
                    println!(
                        "\t{} {}",
                        emoji::X,
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
                        "assert" | "error" => format!("{} {}", emoji::X, styles::warning(messages)),
                        "warn" => format!("{} {}", emoji::WARN, styles::highlight(messages)),
                        "trace" | "debug" => {
                            format!("{}{}", emoji::MICROSCOPE, styles::cyan(messages))
                        }
                        _ => format!("{} {}", emoji::FILES, styles::bold(messages)),
                    };

                    println!("\t{}", output);
                });
            }

            Ok(Response::new(Body::from("Success")))
        }
        _ => {
            let mut bad_request = Response::default();
            *bad_request.status_mut() = StatusCode::BAD_REQUEST;
            Ok(bad_request)
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct LogResponse {
    outcome: String,
    script_name: Option<String>,
    // todo: wtf gets served here
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
    // headers: bruh,
    // cf: RequestEventCfData, // lol
}
