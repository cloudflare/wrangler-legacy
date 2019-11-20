use std::net::{SocketAddr, ToSocketAddrs};
use std::str;

use chrono::prelude::*;

use hyper2::client::{HttpConnector, ResponseFuture};
use hyper2::header::{HeaderMap, HeaderName, HeaderValue, UPGRADE};
use hyper2::service::{make_service_fn, service_fn};
use hyper2::upgrade::Upgraded;
use hyper2::{Body, Client, Request, Response, Server, StatusCode, Uri};

use hyper_tls2::HttpsConnector;

use failure::format_err;

use tokio::io::{AsyncReadExt, AsyncWriteExt};

use uuid::adapter::Simple;
use uuid::Uuid;

use url::Url;

use crate::commands;
use crate::commands::preview::upload;

use crate::settings::global_user::GlobalUser;
use crate::settings::target::Target;

use crate::terminal::emoji;

const PREVIEW_HOST: &str = "rawhttp.cloudflareworkers.com";
const HEADER_PREFIX: &str = "cf-ew-raw-";

#[derive(Clone)]
struct ServerConfig {
    host: String,
    listening_address: SocketAddr,
    is_https: bool,
}

impl ServerConfig {
    pub fn new(
        host: Option<&str>,
        ip: Option<&str>,
        port: Option<&str>,
    ) -> Result<Self, failure::Error> {
        let port = port.unwrap_or("8000");

        let try_address = match ip {
            Some(ip) => format!("{}:{}", ip, port),
            None => format!("localhost:{}", port),
        };

        let mut address_iter = try_address.to_socket_addrs()?;

        let listening_address = address_iter
            .next()
            .ok_or_else(|| format_err!("Could not parse address {}", try_address))?;

        let host = host.unwrap_or("https://example.com").to_string();

        let parsed_url = match Url::parse(&host) {
            Ok(host) => Ok(host),
            Err(_) => Url::parse(&format!("https://{}", host)),
        }?;

        let scheme = parsed_url.scheme();
        if scheme != "http" && scheme != "https" {
            failure::bail!("Your host scheme must be either http or https")
        }
        let is_https = scheme == "https";

        let host = parsed_url.host_str().ok_or(format_err!("Invalid host, accepted formats are example.com, http://example.com, or https://example.com"))?.to_string();

        Ok(ServerConfig {
            listening_address,
            host,
            is_https,
        })
    }

    fn listening_address_as_string(&self) -> String {
        self.listening_address
            .to_string()
            .replace("[::1]", "localhost")
    }
}

async fn client_upgraded_io(mut upgraded: Upgraded) -> Result<(), failure::Error> {
    // We've gotten an upgraded connection that we can read
    // and write directly on. Let's start out 'foobar' protocol.
    upgraded.write_all(b"foo=bar").await?;
    println!("client[foobar] sent");

    let mut vec = Vec::new();
    upgraded.read_to_end(&mut vec).await?;
    println!("client[foobar] recv: {:?}", str::from_utf8(&vec));

    Ok(())
}

async fn client_upgrade_request(session_id: String) -> Result<(), failure::Error> {
    let req = Request::builder()
        .uri(format!(
            "https://rawhttp.cloudflareworkers.com/inspect/{}",
            session_id
        ))
        .header(UPGRADE, "websocket")
        .body(Body::empty())
        .unwrap();

    println!("{:#?}", req);

    let https = HttpsConnector::new().expect("TLS initialization failed");
    let res = Client::builder()
        .build::<_, Body>(https)
        .request(req)
        .await?;
    if res.status() != StatusCode::SWITCHING_PROTOCOLS {
        panic!("Our server didn't upgrade: {}", res.status());
    }

    match res.into_body().on_upgrade().await {
        Ok(upgraded) => {
            if let Err(e) = client_upgraded_io(upgraded).await {
                eprintln!("client foobar io error: {}", e)
            };
        }
        Err(e) => eprintln!("upgrade error: {}", e),
    }

    Ok(())
}

pub async fn dev_server(
    target: Target,
    user: Option<GlobalUser>,
    host: Option<&str>,
    port: Option<&str>,
    ip: Option<&str>,
) -> Result<(), failure::Error> {
    commands::build(&target)?;
    let server_config = ServerConfig::new(host, ip, port)?;
    let https = HttpsConnector::new().expect("TLS initialization failed");
    let client = Client::builder().build::<_, Body>(https);

    let session_id = get_session_id();
    let preview_id = get_preview_id(target, user, &server_config, session_id)?;
    let listening_address = server_config.listening_address.clone();
    let listening_address_string = server_config.listening_address_as_string();

    let make_service = make_service_fn(move |_| {
        let client = client.clone();
        let preview_id = preview_id.to_owned();
        let server_config = server_config.clone();
        async move {
            Ok::<_, failure::Error>(service_fn(move |req| {
                let client = client.to_owned();
                let preview_id = preview_id.to_owned();
                let server_config = server_config.clone();
                async move {
                    let resp = preview_request(req, client, preview_id, server_config).await?;

                    let (mut parts, body) = resp.into_parts();

                    let mut headers = HeaderMap::new();

                    for header in &parts.headers {
                        let (name, value) = header;
                        let name = name.as_str();
                        if name.starts_with(HEADER_PREFIX) {
                            let header_name = &name[HEADER_PREFIX.len()..];
                            // TODO: remove unwrap
                            let header_name =
                                HeaderName::from_bytes(header_name.as_bytes()).unwrap();
                            headers.insert(header_name, value.clone());
                        }
                    }
                    parts.headers = headers;

                    let resp = Response::from_parts(parts, body);
                    Ok::<_, failure::Error>(resp)
                }
            }))
        }
    });

    let server = Server::bind(&listening_address).serve(make_service);

    println!(
        "{} Listening on http://{}",
        emoji::EAR,
        listening_address_string
    );
    hyper2::rt::spawn(async {
        if let Err(e) = server.await {
            eprintln!("server error: {}", e);
        }
    });

    let request = client_upgrade_request(session_id.to_string());

    if let Err(e) = request.await {
        eprintln!("client error: {}", e);
    }

    Ok(())
}

fn get_preview_url(path_string: &str) -> Result<Uri, http::uri::InvalidUri> {
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

    let mut headers: HeaderMap = HeaderMap::new();

    for header in &parts.headers {
        let (name, value) = header;
        let forward_header = format!("{}{}", HEADER_PREFIX, name);
        // TODO: remove unwrap
        let header_name = HeaderName::from_bytes(forward_header.as_bytes()).unwrap();
        headers.insert(header_name, value.clone());
    }
    parts.headers = headers;

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
    session: Simple,
) -> Result<String, failure::Error> {
    let verbose = true;
    let sites_preview = false;
    let script_id: String = upload(&mut target, user.as_ref(), sites_preview, verbose)?;
    Ok(format!(
        "{}{}{}{}",
        &script_id, session, server_config.is_https as u8, server_config.host
    ))
}

fn get_session_id() -> Simple {
    Uuid::new_v4().to_simple()
}
