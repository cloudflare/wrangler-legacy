#![deny(warnings)]
extern crate hyper;
extern crate hyper_tls;
extern crate pretty_env_logger;
extern crate url;

use chrono::prelude::*;

use hyper::header::{HeaderName, HeaderValue};
use hyper::rt::{self, Future};
use hyper::service::service_fn;
use hyper::{Client, Server};

use hyper_tls::HttpsConnector;

use uuid::Uuid;

use url::Url;

use crate::settings::global_user::GlobalUser;
use crate::settings::target::Target;

use crate::commands;
use crate::commands::preview::upload;

const PREVIEW_HOST: &str = "rawhttp.cloudflareworkers.com";

pub fn proxy(
    mut target: Target,
    user: Option<GlobalUser>,
    host: Option<&str>,
    port: Option<&str>,
) -> Result<(), failure::Error> {
    commands::build(&target)?;
    let port: u16 = match port {
        Some(port) => port.to_string().parse(),
        None => Ok(3000),
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
        failure::bail!("Your host must be either http or https")
    }
    let is_https = scheme == "https";
    let host_str: Option<&str> = parsed_url.host_str();
    let host: Result<String, failure::Error> = if let Some(host_str) = host_str {
        Ok(host_str.to_string())
    } else {
        failure::bail!("Invalid host, accepted formats are http://example.com or example.com")
    };
    let host = host?;
    let listening_address = ([127, 0, 0, 1], port).into();

    let https = HttpsConnector::new(4).expect("TLS initialization failed");
    let client_main = Client::builder().build::<_, hyper::Body>(https);

    let session = Uuid::new_v4().to_simple();
    let verbose = true;
    let sites_preview = false;
    let script_id: String = upload(&mut target, user.as_ref(), sites_preview, verbose)?;

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
            let uri_path_and_query = req.uri().path_and_query().map(|x| x.as_str()).unwrap_or("");
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

    let server = Server::bind(&listening_address)
        .serve(new_service)
        .map_err(|e| eprintln!("server error: {}", e));

    println!("Serving HTTP on http://{}", listening_address);

    rt::run(server);
    Ok(())
}
