#![deny(warnings)]
extern crate hyper;
extern crate hyper_tls;
extern crate pretty_env_logger;

// use chrono::prelude::*;

use hyper::rt::{self, Future};
use hyper::service::service_fn;
use hyper::{Client, Server};

use hyper_tls::HttpsConnector;

pub fn proxy(port: Option<&str>) -> Result<(), failure::Error> {
    let port: u16 = match port {
        Some(port) => port.to_string().parse(),
        None => Ok(3000),
    }?;
    let host = Some("example.com"); // TODO: make this an arg
    let _ = match host {
        Some(host) => host,
        None => "example.com",
    };
    let listening_address = ([127, 0, 0, 1], port).into();

    let https = HttpsConnector::new(4).expect("TLS initialization failed");
    let client_main = Client::builder().build::<_, hyper::Body>(https);

    // new_service is run for each connection, creating a 'service'
    // to handle requests for that specific connection.
    let new_service = move || {
        let client = client_main.clone();
        // This is the `Service` that will handle the connection.
        // `service_fn_ok` is a helper to convert a function that
        // returns a Response into a `Service`.
        service_fn(move |mut req| {
            // let uri_path_and_query = req.uri().path_and_query().map(|x| x.as_str()).unwrap_or("");
            // let uri_string = format!(
            //     "https://rawhttp.cloudflareworkers.com{}",
            //     uri_path_and_query
            // );

            let uri = "https://hello.avery.workers.dev"
                .parse::<hyper::Uri>()
                .unwrap();
            println!("{:#?}", uri);
            // let method = req.method().to_string();
            // let path = uri_path_and_query.to_string();

            // let now: DateTime<Local> = Local::now();
            *req.uri_mut() = uri;
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
        })
    };

    let server = Server::bind(&listening_address)
        .serve(new_service)
        .map_err(|e| eprintln!("server error: {}", e));

    println!("Serving HTTP on http://{}", listening_address);

    rt::run(server);
    Ok(())
}
