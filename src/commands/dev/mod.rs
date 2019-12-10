mod server_config;
use server_config::ServerConfig;

use chrono::prelude::*;

use hyper::client::{HttpConnector, ResponseFuture};
use hyper::header::{HeaderName, HeaderValue};
use hyper::service::{make_service_fn, service_fn};
use hyper::{Body, Client, Request, Server, Uri};
use hyper::http::uri::InvalidUri;

use hyper_tls::HttpsConnector;

use uuid::Uuid;

use crate::commands;
use crate::commands::preview::upload;

use crate::settings::global_user::GlobalUser;
use crate::settings::target::Target;

use crate::terminal::emoji;

const PREVIEW_HOST: &str = "rawhttp.cloudflareworkers.com";

pub async fn dev(
    target: Target,
    user: Option<GlobalUser>,
    host: Option<&str>,
    port: Option<&str>,
    ip: Option<&str>,
) -> Result<(), failure::Error> {
    commands::build(&target)?;
    let server_config = ServerConfig::new(host, ip, port)?;

    // set up https client to connect to the preview service
    let https = HttpsConnector::new();
    let client = Client::builder().build::<_, Body>(https);

    let preview_id = get_preview_id(target, user, &server_config)?;
    let listening_address = server_config.listening_address.clone();

    // create a closure that hyper will use later to handle HTTP requests
    let make_service = make_service_fn(move |_| {
        let client = client.clone();
        let preview_id = preview_id.to_owned();
        let server_config = server_config.clone();
        async move {
            Ok::<_, failure::Error>(service_fn(move |req| {
                preview_request(
                    req,
                    client.to_owned(),
                    preview_id.to_owned(),
                    server_config.clone(),
                )
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

fn get_path_as_str(uri: Uri) -> String {
    uri.path_and_query()
        .map(|x| x.as_str())
        .unwrap_or("")
        .to_string()
}

fn preview_request(
    req: Request<Body>,
    client: Client<HttpsConnector<HttpConnector>>,
    preview_id: String,
    server_config: ServerConfig,
) -> ResponseFuture {
    let (mut parts, body) = req.into_parts();

    let path = get_path_as_str(parts.uri);
    let method = parts.method.to_string();
    let now: DateTime<Local> = Local::now();
    let preview_id = &preview_id;

    // TODO: remove unwrap
    parts.uri = get_preview_url(&path).unwrap();
    parts.headers.insert(
        HeaderName::from_static("host"),
        HeaderValue::from_static(PREVIEW_HOST),
    );

    // TODO: remove unwrap
    parts.headers.insert(
        HeaderName::from_static("cf-ew-preview"),
        HeaderValue::from_str(preview_id).unwrap(),
    );

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

fn get_preview_id(
    mut target: Target,
    user: Option<GlobalUser>,
    server_config: &ServerConfig,
) -> Result<String, failure::Error> {
    let session = Uuid::new_v4().to_simple();
    let verbose = true;
    let sites_preview = false;
    let script_id: String = upload(&mut target, user.as_ref(), sites_preview, verbose)?;
    Ok(format!(
        "{}{}{}{}",
        &script_id,
        session,
        server_config.host.is_https() as u8,
        server_config.host
    ))
}
