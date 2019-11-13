#![deny(warnings)]
extern crate hyper;
extern crate pretty_env_logger;

use hyper::rt::{self, Future};
use hyper::service::service_fn;
use hyper::{Client, Server};
use std::net::SocketAddr;

pub fn proxy(port: Option<&str>) -> Result<(), failure::Error> {
    let port: u16 = match port {
        Some(port) => port.to_string().parse(),
        None => Ok(3000),
    }?;
    let listening_address = ([127, 0, 0, 1], port).into();
    let proxy_to_address: SocketAddr = ([127, 0, 0, 1], 3001).into();

    let client_main = Client::new();

    let proxy_to_address_clone = proxy_to_address.clone();
    // new_service is run for each connection, creating a 'service'
    // to handle requests for that specific connection.
    let new_service = move || {
        let client = client_main.clone();
        // This is the `Service` that will handle the connection.
        // `service_fn_ok` is a helper to convert a function that
        // returns a Response into a `Service`.
        service_fn(move |mut req| {
            let uri_path_and_query = req.uri().path_and_query().map(|x| x.as_str()).unwrap_or("");
            let uri_string = format!("http://{}{}", proxy_to_address_clone, uri_path_and_query);
            let uri = uri_string.parse().unwrap();
            *req.uri_mut() = uri;
            client.request(req)
        })
    };

    let server = Server::bind(&listening_address)
        .serve(new_service)
        .map_err(|e| eprintln!("server error: {}", e));

    println!("Your worker is emulated on http://{}", listening_address);

    rt::run(server);
    Ok(())
}
