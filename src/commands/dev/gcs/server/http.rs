use super::preview_request;
use crate::commands::dev::gcs::headers::destructure_response;
use crate::commands::dev::server_config::ServerConfig;
use crate::commands::dev::utils::{get_path_as_str, rewrite_redirect};
use crate::terminal::emoji;

use std::sync::{Arc, Mutex};

use anyhow::Result;
use chrono::prelude::*;
use hyper::service::{make_service_fn, service_fn};
use hyper::{Body, Client as HyperClient, Request, Response, Server};
use hyper_rustls::HttpsConnector;

/// performs all logic that takes an incoming request
/// and routes it to the Workers runtime preview service
pub async fn http(server_config: ServerConfig, preview_id: Arc<Mutex<String>>) -> Result<()> {
    // set up https client to connect to the preview service
    let https = HttpsConnector::with_native_roots();
    let client = HyperClient::builder().build::<_, Body>(https);

    let listening_address = server_config.listening_address;

    // create a closure that hyper will use later to handle HTTP requests
    // this takes care of sending an incoming request along to
    // the uploaded Worker script and returning its response
    let make_service = make_service_fn(move |_| {
        let client = client.to_owned();
        let server_config = server_config.to_owned();
        let preview_id = preview_id.to_owned();
        async move {
            Ok::<_, anyhow::Error>(service_fn(move |req| {
                let client = client.to_owned();
                let server_config = server_config.to_owned();
                let preview_id = preview_id.lock().unwrap().to_owned();
                let version = req.version();

                // record the time of the request
                let now: DateTime<Local> = Local::now();

                // split the request into parts so we can read
                // what it contains and display in logs
                let (parts, body) = req.into_parts();
                let local_host = format!(
                    "{}:{}",
                    server_config.listening_address.ip().to_string(),
                    server_config.listening_address.port().to_string()
                );

                let req_method = parts.method.to_string();

                // parse the path so we can send it to the preview service
                // we don't want to send "localhost:8787/path", just "/path"
                let path = get_path_as_str(&parts.uri);

                async move {
                    // send the request to the preview service
                    let resp = preview_request(
                        Request::from_parts(parts, body),
                        client,
                        preview_id.to_owned(),
                    )
                    .await?;
                    let (mut parts, body) = resp.into_parts();

                    // format the response for the user
                    destructure_response(&mut parts)?;
                    let mut resp = Response::from_parts(parts, body);
                    rewrite_redirect(
                        &mut resp,
                        &server_config.host.to_string(),
                        &local_host,
                        false,
                    );

                    // print information about the response
                    // [2020-04-20 15:25:54] GET example.com/ HTTP/1.1 200 OK
                    println!(
                        "[{}] {} {}{} {:?} {}",
                        now.format("%Y-%m-%d %H:%M:%S"),
                        req_method,
                        server_config.host,
                        path,
                        version,
                        resp.status()
                    );
                    Ok::<_, anyhow::Error>(resp)
                }
            }))
        }
    });

    let server = Server::bind(&listening_address).serve(make_service);
    println!(
        "{} Listening on http://{}",
        emoji::EAR,
        listening_address.to_string()
    );
    if let Err(e) = server.await {
        eprintln!("server error: {}", e);
    }
    Ok(())
}
