use std::net::{SocketAddr, ToSocketAddrs};

use hyper2::client::{HttpConnector, ResponseFuture};
use hyper2::error::Error;
use hyper2::header::{HeaderValue, InvalidHeaderValue};
use hyper2::service::{make_service_fn, service_fn};
use hyper2::{Body, Client, Request, Response, Server};

use hyper_tls2::HttpsConnector;

use failure::format_err;

use futures_util::TryStreamExt;

use uuid::Uuid;

use url::Url;

use crate::settings::global_user::GlobalUser;
use crate::settings::target::Target;

use crate::commands::preview::upload;

const PREVIEW_HOST: &str = "rawhttp.cloudflareworkers.com";

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
        let port: &str = match port {
            Some(port) => port,
            None => "8000",
        };

        let try_address = match ip {
            Some(ip) => format!("{}:{}", ip, port),
            None => format!("localhost:{}", port),
        };

        let mut address_iter = try_address.to_socket_addrs()?;

        let listening_address = match address_iter.next() {
            Some(ip) => Ok(ip),
            None => Err(format_err!("Could not parse address {}", try_address)),
        }?;

        let host: String = match host {
            Some(host) => host.to_string(),
            None => "https://example.com".to_string(),
        };

        let parsed_url = match Url::parse(&host) {
            Ok(host) => Ok(host),
            Err(_) => Url::parse(&format!("https://{}", host)),
        }?;

        let scheme: &str = parsed_url.scheme();
        if scheme != "http" && scheme != "https" {
            failure::bail!("Your host scheme must be either http or https")
        }
        let is_https = scheme == "https";

        let host = match parsed_url.host_str() {
            Some(host_str) => Ok(host_str.to_string()),
            None => Err(format_err!(
                "Invalid host, accepted formats are http://example.com or example.com"
            )),
        }?;

        let proxy = ProxyConfig {
            listening_address,
            host,
            is_https,
        };

        Ok(proxy)
    }

    fn get_listening_address_as_str(&self) -> String {
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
    let proxy_config = ProxyConfig::new(host, ip, port)?;
    let https = HttpsConnector::new().expect("TLS initialization failed");
    let client = Client::builder().build::<_, Body>(https);

    let preview_id = get_preview_id(target, user, &proxy_config)?;
    let make_service = make_service_fn(move |_| {
        let client = client.clone();
        let preview_id = preview_id.to_owned();
        async move {
            Ok::<_, Error>(service_fn(move |req| {
                preview_request(req, client.to_owned(), preview_id.to_owned())
            }))
        }
    });

    let server = Server::bind(&proxy_config.listening_address).serve(make_service);

    let listening_address_str = proxy_config.get_listening_address_as_str();
    println!("Listening on http://{}", listening_address_str);
    if let Err(e) = server.await {
        eprintln!("server error: {}", e);
    }
    Ok(())
}

fn preview_request(
    req: Request<Body>,
    client: Client<HttpsConnector<HttpConnector>>,
    preview_id: String,
) -> ResponseFuture {
    let (mut parts, body) = req.into_parts();
    let mut req = Request::from_parts(parts, body);

    // let uri_path_and_query =
    //     req.uri().path_and_query().map(|x| x.as_str()).unwrap_or("");
    // let uri_string = format!("https://{}{}", PREVIEW_HOST, uri_path_and_query);

    // let uri = uri_string.parse::<hyper::Uri>().unwrap();
    // let method = req.method().to_string();
    // let path = uri_path_and_query.to_string();

    // let now: DateTime<Local> = Local::now();
    // *req.uri_mut() = uri;
    // let headers = req.headers_mut();
    // headers.insert(
    //     HeaderName::from_static("host"),
    //     HeaderValue::from_static(PREVIEW_HOST),
    // );
    // println!("{:?}", headers);
    // let preview_id = HeaderValue::from_str(&format!(
    //     "{}{}{}{}",
    //     &script_id, session, is_https as u8, host
    // ));

    // if let Ok(preview_id) = preview_id {
    //     req.headers_mut()
    //         .insert(HeaderName::from_static("cf-ew-preview"), preview_id);
    //     println!(
    //         "[{}] \"{} {}{} {:?}\"",
    //         now.format("%Y-%m-%d %H:%M:%S"),
    //         method,
    //         host,
    //         path,
    //         req.version()
    //     );
    //     client.request(req)
    // }
    // })

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
