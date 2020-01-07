mod server_config;
mod socket;
use server_config::ServerConfig;
mod headers;
use headers::{destructure_response, structure_request};

use std::thread;
use std::sync::{mpsc, Mutex, Arc};

use chrono::prelude::*;

use hyper::client::{HttpConnector, ResponseFuture};
use hyper::header::{HeaderName, HeaderValue};

use hyper::http::uri::InvalidUri;
use hyper::service::{make_service_fn, service_fn};
use hyper::{Body, Client as HyperClient, Request, Response, Server, Uri};

use hyper_tls::HttpsConnector;

use tokio::runtime::Runtime as TokioRuntime;

use uuid::Uuid;

use crate::commands;
use crate::commands::preview::upload;

use crate::settings::global_user::GlobalUser;
use crate::settings::toml::Target;

use crate::terminal::emoji;

const PREVIEW_HOST: &str = "rawhttp.cloudflareworkers.com";

pub fn dev(
    target: Target,
    user: Option<GlobalUser>,
    watch: bool,
    host: Option<&str>,
    port: Option<&str>,
    ip: Option<&str>,
) -> Result<(), failure::Error> {
    commands::build(&target)?;
    let server_config = ServerConfig::new(host, ip, port)?;
    let session_id = get_session_id()?;
    let preview_id = get_preview_id(target.clone(), user.clone(), &server_config, &session_id)?;
    let preview_id = Arc::new(Mutex::new(preview_id));

    // create a new thread to listen for devtools messages
    thread::spawn(move || socket::listen(&session_id));
    
    { // create an artificial scope to create new preview_id Arc
        let preview_id = preview_id.clone();
        let server_config = server_config.clone();
        thread::spawn(move || watch_for_changes(target, user, &server_config, Arc::clone(&preview_id), true));
    }

    // spawn tokio runtime on the main thread to handle incoming HTTP requests
    let mut runtime = TokioRuntime::new()?;
    runtime.block_on(serve(server_config, Arc::clone(&preview_id)))?;

    Ok(())
}

async fn serve(server_config: ServerConfig, preview_id: Arc<Mutex<String>>) -> Result<(), failure::Error> {
    // set up https client to connect to the preview service
    let https = HttpsConnector::new();
    let client = HyperClient::builder().build::<_, Body>(https);

    let listening_address = server_config.listening_address.clone();
    // create a closure that hyper will use later to handle HTTP requests
    let make_service = make_service_fn(move |_| {
        let client = client.to_owned();
        let preview_id = preview_id.to_owned();
        let server_config = server_config.to_owned();
        async move {
            Ok::<_, failure::Error>(service_fn(move |req| {
                let client = client.to_owned();
                let preview_id = preview_id.lock().unwrap().to_owned();
                let server_config = server_config.to_owned();
                async move {
                    let resp = preview_request(req, client, preview_id.to_owned(), server_config).await?;
                    let (mut parts, body) = resp.into_parts();

                    destructure_response(&mut parts)?;
                    let resp = Response::from_parts(parts, body);
                    Ok::<_, failure::Error>(resp)
                }
            }))
        }
    });

    let server = Server::bind(&listening_address.address).serve(make_service);
    println!("{} Listening on http://{}", emoji::EAR, listening_address);
    if let Err(e) = server.await {
        eprintln!("server error: {}", e);
    }
    Ok(())
}

fn get_preview_url(path_string: &str) -> Result<Uri, InvalidUri> {
    format!("https://{}{}", PREVIEW_HOST, path_string).parse()
}

fn get_path_as_str(uri: &Uri) -> String {
    uri.path_and_query()
        .map(|x| x.as_str())
        .unwrap_or("")
        .to_string()
}

fn preview_request(
    req: Request<Body>,
    client: HyperClient<HttpsConnector<HttpConnector>>,
    preview_id: String,
    server_config: ServerConfig,
) -> ResponseFuture {
    let (mut parts, body) = req.into_parts();

    let path = get_path_as_str(&parts.uri);
    let method = parts.method.to_string();
    let now: DateTime<Local> = Local::now();
    let preview_id = &preview_id;
    println!("request id: {:?}", preview_id);

    structure_request(&mut parts);

    parts.headers.insert(
        HeaderName::from_static("host"),
        HeaderValue::from_static(PREVIEW_HOST),
    );

    parts.headers.insert(
        HeaderName::from_static("cf-ew-preview"),
        HeaderValue::from_str(preview_id).expect("Could not create header for preview id"),
    );

    parts.uri = get_preview_url(&path).expect("Could not get preview url");

    let req = Request::from_parts(parts, body);

    
    println!(
        "[{}] \"{} {}{} {:?}\"",
        now.format("%Y-%m-%d %H:%M:%S"),
        method,
        server_config.host,
        path,
        req.version()
    );
    client.request(req)
}

fn get_session_id() -> Result<String, failure::Error> {
    Ok(Uuid::new_v4().to_simple().to_string())
}

fn get_preview_id(
    mut target: Target,
    user: Option<GlobalUser>,
    server_config: &ServerConfig,
    session_id: &str,
) -> Result<String, failure::Error> {
    let verbose = true;
    let sites_preview = false;
    let script_id: String = upload(&mut target, user.as_ref(), sites_preview, verbose)?;
    Ok(format!(
        "{}{}{}{}",
        &script_id,
        session_id,
        server_config.host.is_https() as u8,
        server_config.host
    ))
}

fn watch_for_changes(
    target: Target,
    user: Option<GlobalUser>,
    server_config: &ServerConfig,
    preview_id: Arc<Mutex<String>>,
    verbose: bool,
) -> Result<(), failure::Error> {
    let sites_preview: bool = target.site.is_some();

    let (tx, rx) = mpsc::channel();
    
    commands::watch_and_build(&target, Some(tx))?;

    while let Ok(_) = rx.recv() {
        let user = user.clone();
        let mut target = target.clone();
        commands::build(&target)?;

        if let Ok(new_id) = upload(&mut target, user.as_ref(), sites_preview, verbose) {
            let mut p = preview_id.lock().unwrap();
            *p = get_preview_id(target, user, server_config, &new_id)?;
        }
    }

    Ok(())
}