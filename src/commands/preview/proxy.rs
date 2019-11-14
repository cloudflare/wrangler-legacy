#![deny(warnings)]
extern crate hyper;
extern crate hyper_tls;
extern crate pretty_env_logger;

// use chrono::prelude::*;

use hyper::header::{HeaderName, HeaderValue};
use hyper::rt::{self, Future};
use hyper::service::service_fn;
use hyper::{Client, Server};

use hyper_tls::HttpsConnector;

use uuid::Uuid;

use crate::settings::global_user::GlobalUser;
use crate::settings::target::Target;

use crate::commands::preview::upload;

const PREVIEW_HOST: &str = "rawhttp.cloudflareworkers.com";

pub fn proxy(
    mut target: Target,
    user: Option<GlobalUser>,
    port: Option<&str>,
) -> Result<(), failure::Error> {
    let port: u16 = match port {
        Some(port) => port.to_string().parse(),
        None => Ok(3000),
    }?;
    let host = Some("example.com"); // TODO: make this an arg
    let host = match host {
        Some(host) => host,
        None => "example.com",
    };
    let listening_address = ([127, 0, 0, 1], port).into();

    let https = HttpsConnector::new(4).expect("TLS initialization failed");
    let client_main = Client::builder().build::<_, hyper::Body>(https);

    let https = true;
    let session = Uuid::new_v4().to_simple();
    let verbose = true;
    let sites_preview = false;
    let script_id: String = upload(&mut target, user.as_ref(), sites_preview, verbose)?;

    // new_service is run for each connection, creating a 'service'
    // to handle requests for that specific connection.
    let new_service = move || {
        let client = client_main.clone();
        let script_id = script_id.clone();
        // This is the `Service` that will handle the connection.
        // `service_fn_ok` is a helper to convert a function that
        // returns a Response into a `Service`.
        service_fn(move |mut req| {
            // let uri_path_and_query = req.uri().path_and_query().map(|x| x.as_str()).unwrap_or("");
            // let uri_string = format!(
            //     "https://rawhttp.cloudflareworkers.com{}",
            //     uri_path_and_query
            // );

            let uri = format!("https://{}", PREVIEW_HOST)
                .parse::<hyper::Uri>()
                .unwrap();
            println!("{:#?}", uri);
            // let method = req.method().to_string();
            // let path = uri_path_and_query.to_string();

            // let now: DateTime<Local> = Local::now();
            *req.uri_mut() = uri;
            req.headers_mut().insert(
                HeaderName::from_static("host"),
                HeaderValue::from_static(PREVIEW_HOST),
            );
            let preview_id = HeaderValue::from_str(&format!(
                "{}{}{}{}",
                &script_id.clone(),
                session,
                https as u8,
                host
            ));

            if let Ok(preview_id) = preview_id {
                req.headers_mut()
                    .insert(HeaderName::from_static("cf-ew-preview"), preview_id);
                // println!(
                //     "[{}] \"{} {}{} {:?}\"",
                //     now.format("%Y-%m-%d %H:%M:%S"),
                //     method,
                //     host,
                //     path,
                //     req.version()
                // );
                dbg!(&req);
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
