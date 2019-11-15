use crate::commands;
use crate::terminal::message;

use std::path::PathBuf;
use std::process::Command;
use std::thread;
use std::str;

use futures::future;
use futures::stream::Stream;
use hyper::{Body, Request, Response, Server, Method, StatusCode};
use hyper::rt::Future;
use hyper::service::service_fn;

pub fn run_cloudflared_start_server() -> Result<(), failure::Error> {
    let tool_name = PathBuf::from("cloudflared");
    // let binary_path = install::install(tool_name, "cloudflare")?.binary(tool_name)?;
    let args = ["tunnel"];

    let command = command(&args, &tool_name);
    let command_name = format!("{:?}", command);

    start_echo_http_server();

    // Likely want to get rid of these printouts.
    message::working("Starting up an Argo Tunnel");
    commands::run(command, &command_name)?;

    thread::sleep(std::time::Duration::from_secs(300));

    Ok(())
}

pub fn start_echo_http_server() {
    // Start HTTP echo server that prints whatever is posted to it.
    let addr = ([127, 0, 0, 1], 8080).into();
    message::working("HTTP Echo server is running on 127.0.0.1:8080");

    let server = Server::bind(&addr)
        .serve(|| service_fn(echo))
        .map_err(|e| eprintln!("server error: {}", e));

    thread::spawn(move || {
        hyper::rt::run(server);
    });
}

fn echo(req: Request<Body>) -> impl Future<Item = Response<Body>, Error = hyper::Error> {
    let (parts, body) = req.into_parts();

    match (parts.method, parts.uri.path()) {
        (Method::POST, "/") => {
            let entire_body = body.concat2();
            let resp = entire_body.map(|body| {
                println!("{:?}", str::from_utf8(&body).unwrap());
                Response::new(Body::from("Success"))
            });
            future::Either::A(resp)
        }
        _ => {
            let body = Body::from("Can only POST to /");
            let mut response = Response::new(body);
            *response.status_mut() = StatusCode::NOT_FOUND;
            let resp = future::ok(response);
            future::Either::B(resp)
        }
    }
    // let mut response = Response::new(Body::empty());

    // match (req.method(), req.uri().path()) {
    //     // (&Method::GET, "/") => {
    //     //     *response.body_mut() = Body::from("Try POSTing data to /echo");
    //     // },
    //     (&Method::POST, "/echo") => {
    //         // let body = req.into_body();
    //         let entire_body = req.into_body().concat2();
    //         let resp = entire_body.map(|body| {
    //             let body = Body::from(format!("Read {} bytes", body.len()));
    //             println!("{:?}", body);
    //             Response::new(body)
    //         });
    //         future::Either::A(resp);
    //         // *response.body_mut() = Body::from("Success")
    //     },
    //     _ => {
    //         // *response.status_mut() = StatusCode::NOT_FOUND;
    //         let body = Body::from("Can only POST to /");
    //         let resp = future::ok(Response::new(body));
    //         future::Either::B(resp)
    //     },
    // };

    // Box::new(future::ok(response))
}

// TODO(gabbi): let's not clumsily copy this from commands/build/mod.rs
pub fn command(args: &[&str], binary_path: &PathBuf) -> Command {
    let mut c = if cfg!(target_os = "windows") {
        let mut c = Command::new("cmd");
        c.arg("/C");
        c.arg(binary_path);
        c
    } else {
        Command::new(binary_path)
    };

    c.args(args);
    c
}