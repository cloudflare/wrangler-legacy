use crate::commands::dev::server_config::ServerConfig;
use crate::commands::dev::utils::get_path_as_str;
use crate::terminal::emoji;

use chrono::prelude::*;
use hyper::client::{HttpConnector, ResponseFuture};
use hyper::header::{HeaderName, HeaderValue};
use hyper::service::{make_service_fn, service_fn};
use hyper::{Body, Client as HyperClient, Request, Server};
use hyper_tls::HttpsConnector;

pub(super) async fn serve(
    server_config: ServerConfig,
    preview_token: String,
    host: String,
) -> Result<(), failure::Error> {
    // set up https client to connect to the preview service
    let https = HttpsConnector::new();
    let client = HyperClient::builder().build::<_, Body>(https);

    let listening_address = server_config.listening_address.clone();

    // create a closure that hyper will use later to handle HTTP requests
    let make_service = make_service_fn(move |_| {
        let client = client.to_owned();
        let preview_token = preview_token.to_owned();
        let host = host.to_owned();

        async move {
            Ok::<_, failure::Error>(service_fn(move |req| {
                let client = client.to_owned();
                let preview_token = preview_token.to_owned();
                let host = host.to_owned();
                let version = req.version();
                let (parts, body) = req.into_parts();
                let req_method = parts.method.to_string();
                let now: DateTime<Local> = Local::now();
                let path = get_path_as_str(&parts.uri);
                async move {
                    let resp = preview_request(
                        Request::from_parts(parts, body),
                        client,
                        preview_token.to_owned(),
                        host.clone(),
                    )
                    .await?;

                    println!(
                        "[{}] {} {}{} {:?} {}",
                        now.format("%Y-%m-%d %H:%M:%S"),
                        req_method,
                        host,
                        path,
                        version,
                        resp.status()
                    );
                    Ok::<_, failure::Error>(resp)
                }
            }))
        }
    });

    let server = Server::bind(&listening_address).serve(make_service);
    println!("{} Listening on http://{}", emoji::EAR, listening_address);
    if let Err(e) = server.await {
        eprintln!("server error: {}", e)
    }
    Ok(())
}

fn preview_request(
    req: Request<Body>,
    client: HyperClient<HttpsConnector<HttpConnector>>,
    preview_token: String,
    host: String,
) -> ResponseFuture {
    let (mut parts, body) = req.into_parts();

    let path = get_path_as_str(&parts.uri);

    parts.headers.insert(
        HeaderName::from_static("host"),
        HeaderValue::from_str(&host).expect("Could not create host header"),
    );

    parts.headers.insert(
        HeaderName::from_static("cf-workers-preview-token"),
        HeaderValue::from_str(&preview_token).expect("Could not create header for preview id"),
    );

    // TODO: figure out how to http _or_ https
    parts.uri = format!("https://{}{}", host, path)
        .parse()
        .expect("Could not construct preview url");

    let req = Request::from_parts(parts, body);

    client.request(req)
}
