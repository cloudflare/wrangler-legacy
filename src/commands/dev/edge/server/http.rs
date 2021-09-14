use super::preview_request;
use crate::commands::dev::utils::{get_path_as_str, rewrite_redirect};
use crate::commands::dev::{Protocol, ServerConfig};
use crate::terminal::emoji;

use std::sync::{Arc, Mutex};

use anyhow::Result;
use chrono::prelude::*;
use hyper::service::{make_service_fn, service_fn};
use hyper::{Body, Client as HyperClient, Request, Server};
use hyper_rustls::HttpsConnector;
use tokio::sync::oneshot::{Receiver, Sender};

pub async fn http(
    server_config: ServerConfig,
    preview_token: Arc<Mutex<String>>,
    host: String,
    upstream_protocol: Protocol,
    shutdown_channel: (Receiver<()>, Sender<()>),
) -> Result<()> {
    // set up https client to connect to the preview service
    let https = HttpsConnector::with_native_roots();
    let client = HyperClient::builder().build::<_, Body>(https);

    let listening_address = server_config.listening_address;

    // create a closure that hyper will use later to handle HTTP requests
    let make_service = make_service_fn(move |_| {
        let client = client.to_owned();
        let preview_token = preview_token.to_owned();
        let host = host.to_owned();
        let server_config = server_config.to_owned();

        async move {
            Ok::<_, anyhow::Error>(service_fn(move |req| {
                let client = client.to_owned();
                let preview_token = preview_token.lock().unwrap().to_owned();
                let host = host.to_owned();
                let version = req.version();
                let (parts, body) = req.into_parts();
                let local_host = format!(
                    "{}:{}",
                    server_config.listening_address.ip().to_string(),
                    server_config.listening_address.port().to_string()
                );
                let req_method = parts.method.to_string();
                let now: DateTime<Local> = Local::now();
                let path = get_path_as_str(&parts.uri);
                async move {
                    let mut resp = preview_request(
                        Request::from_parts(parts, body),
                        client,
                        preview_token.to_owned(),
                        host.clone(),
                        upstream_protocol,
                    )
                    .await?;

                    rewrite_redirect(&mut resp, &host, &local_host, false);

                    println!(
                        "[{}] {} {}{} {:?} {}",
                        now.format("%Y-%m-%d %H:%M:%S"),
                        req_method,
                        host,
                        path,
                        version,
                        resp.status()
                    );
                    Ok::<_, anyhow::Error>(resp)
                }
            }))
        }
    });

    let (rx, tx) = shutdown_channel;
    let server = Server::bind(&listening_address)
        .serve(make_service)
        .with_graceful_shutdown(async {
            rx.await.expect("Could not receive shutdown initiation");
        });
    println!("{} Listening on http://{}", emoji::EAR, listening_address);

    if let Err(e) = server.await {
        eprintln!("{}", e);
    }
    if let Err(e) = tx.send(()) {
        log::error!("Could not acknowledge dev http listener shutdown: {:?}", e);
    }

    Ok(())
}
