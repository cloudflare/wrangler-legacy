use std::net::{SocketAddr, ToSocketAddrs};

use chrono::prelude::*;

use hyper2::client::{HttpConnector, ResponseFuture};
use hyper2::header::{HeaderName, HeaderValue};
use hyper2::service::{make_service_fn, service_fn};
use hyper2::{Body, Client, Request, Server, Uri};

use hyper_tls2::HttpsConnector;

use failure::format_err;

use uuid::Uuid;

use url::Url;

use crate::settings::global_user::GlobalUser;
use crate::settings::target::Target;

use crate::commands;
use crate::commands::preview::upload;

const PREVIEW_HOST: &str = "rawhttp.cloudflareworkers.com";

#[derive(Clone)]
struct ProxyConfig {
    host: String,
    listening_address: SocketAddr,
    is_https: bool,
}

impl ProxyConfig {
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

        Ok(ProxyConfig {
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

pub async fn proxy(
    target: Target,
    user: Option<GlobalUser>,
    host: Option<&str>,
    port: Option<&str>,
    ip: Option<&str>,
) -> Result<(), failure::Error> {
    commands::build(&target)?;
    let proxy_config = ProxyConfig::new(host, ip, port)?;
    let https = HttpsConnector::new().expect("TLS initialization failed");
    let client = Client::builder().build::<_, Body>(https);

    let preview_id = get_preview_id(target, user, &proxy_config)?;
    let listening_address = &proxy_config.listening_address.clone();
    let listening_address_string = proxy_config.listening_address_as_string();

    let make_service = make_service_fn(move |_| {
        let client = client.clone();
        let preview_id = preview_id.to_owned();
        let proxy_config = proxy_config.clone();
        async move {
            Ok::<_, failure::Error>(service_fn(move |req| {
                preview_request(
                    req,
                    client.to_owned(),
                    preview_id.to_owned(),
                    proxy_config.clone(),
                )
            }))
        }
    });

    let server = Server::bind(listening_address).serve(make_service);
    println!("Listening on http://{}", listening_address_string);
    if let Err(e) = server.await {
        eprintln!("server error: {}", e);
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
    proxy_config: ProxyConfig,
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
        proxy_config.host,
        path,
        req.version()
    );
    client.request(req)
}

fn get_preview_id(
    mut target: Target,
    user: Option<GlobalUser>,
    proxy_config: &ProxyConfig,
) -> Result<String, failure::Error> {
    let session = Uuid::new_v4().to_simple();
    let verbose = true;
    let sites_preview = false;
    let script_id: String = upload(&mut target, user.as_ref(), sites_preview, verbose)?;
    Ok(format!(
        "{}{}{}{}",
        &script_id, session, proxy_config.is_https as u8, proxy_config.host
    ))
}
