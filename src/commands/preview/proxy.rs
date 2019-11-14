#![deny(warnings)]
extern crate hyper;
extern crate hyper_tls;
extern crate pretty_env_logger;
extern crate url;

use std::net::{SocketAddr, ToSocketAddrs};

use chrono::prelude::*;

use hyper::header::{HeaderName, HeaderValue};
use hyper::rt::{self, Future};
use hyper::service::service_fn;
use hyper::{Client, Server};

use hyper_tls::HttpsConnector;

use failure::format_err;

use uuid::Uuid;

use url::Url;

use crate::settings::global_user::GlobalUser;
use crate::settings::target::Target;

use crate::commands::preview::upload;

const PREVIEW_HOST: &str = "rawhttp.cloudflareworkers.com";

struct Proxy {
    host: String,
    listening_address: SocketAddr,
    is_https: bool,
}

impl Proxy {
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

        let proxy = Proxy {
            listening_address,
            host,
            is_https,
        };

        Ok(proxy)
    }

    pub fn start(
        &self,
        mut target: Target,
        user: Option<GlobalUser>,
    ) -> Result<(), failure::Error> {
        let https = HttpsConnector::new(4).expect("TLS initialization failed");
        let client_main = Client::builder().build::<_, hyper::Body>(https);

        let session = Uuid::new_v4().to_simple();
        let verbose = true;
        let sites_preview = false;
        let script_id: String = upload(&mut target, user.as_ref(), sites_preview, verbose)?;
        let host = self.host.clone();
        let is_https = self.is_https.clone();
        let listening_address_str = self
            .listening_address
            .to_string()
            .replace("[::1]", "localhost");

        // new_service is run for each connection, creating a 'service'
        // to handle requests for that specific connection.
        let new_service = move || {
            let client = client_main.clone();
            let script_id = script_id.clone();
            let host = host.clone();
            // This is the `Service` that will handle the connection.
            // `service_fn_ok` is a helper to convert a function that
            // returns a Response into a `Service`.
            service_fn(move |mut req| {
                let uri_path_and_query =
                    req.uri().path_and_query().map(|x| x.as_str()).unwrap_or("");
                let uri_string = format!("https://{}{}", PREVIEW_HOST, uri_path_and_query);

                let uri = uri_string.parse::<hyper::Uri>().unwrap();
                let method = req.method().to_string();
                let path = uri_path_and_query.to_string();

                let now: DateTime<Local> = Local::now();
                *req.uri_mut() = uri;
                req.headers_mut().insert(
                    HeaderName::from_static("host"),
                    HeaderValue::from_static(PREVIEW_HOST),
                );
                let preview_id = HeaderValue::from_str(&format!(
                    "{}{}{}{}",
                    &script_id, session, is_https as u8, host
                ));

                if let Ok(preview_id) = preview_id {
                    req.headers_mut()
                        .insert(HeaderName::from_static("cf-ew-preview"), preview_id);
                    println!(
                        "[{}] \"{} {}{} {:?}\"",
                        now.format("%Y-%m-%d %H:%M:%S"),
                        method,
                        host,
                        path,
                        req.version()
                    );
                    client.request(req)
                } else {
                    client.request(req)
                }
            })
        };

        let server = Server::bind(&self.listening_address)
            .serve(new_service)
            .map_err(|e| eprintln!("server error: {}", e));

        println!("Listening on http://{}", listening_address_str);

        rt::run(server);
        Ok(())
    }
}

pub fn proxy(
    target: Target,
    user: Option<GlobalUser>,
    host: Option<&str>,
    port: Option<&str>,
    ip: Option<&str>,
) -> Result<(), failure::Error> {
    let proxy = Proxy::new(host, ip, port)?;
    proxy.start(target, user)
}
