use crate::commands;
use crate::http;
use crate::settings::global_user::GlobalUser;
use crate::settings::toml::Target;
use crate::terminal::message;

use cloudflare::endpoints::workers::{CreateTail, CreateTailHeartbeat, CreateTailParams};
// use cloudflare::framework::apiclient::ApiClient
use cloudflare::framework::{
    async_api, async_api::ApiClient};
use cloudflare::framework::{HttpApiClient, HttpApiClientConfig};

use std::path::PathBuf;
use std::process::Command;
use std::str;
use std::thread;
use std::time::Duration;

use hyper::service::{make_service_fn, service_fn};
use hyper::{Body, Method, Request, Response, Server, StatusCode};
use tokio;
use tokio::runtime::Runtime as TokioRuntime;
use reqwest;
use regex::Regex;

pub fn start_tail(target: &Target, user: &GlobalUser) -> Result<(), failure::Error> {
    // do block until async finish here.
    let mut runtime = TokioRuntime::new()?;
    // todo: get rid of this gross clone. Maybe just default to static lifetime.
    runtime.block_on(run_cloudflared_start_server(target.clone(), user.clone()))
}

async fn run_cloudflared_start_server(
    target: Target,
    user: GlobalUser,
) -> Result<(), failure::Error> {
    // need to make create tail API call here and also start a thread for API heartbeat calls
    // (these can be in the same thread but we'll need to communicate the argo tunnel info to
    // the API thread). We can use a channel to transfer this info?
    let res = tokio::try_join!(
        tokio::spawn(async move { enable_tailing_start_heartbeat(&target, &user).await }),
        tokio::spawn(async move { start_log_collection_http_server().await }),
        tokio::spawn(async move { start_argo_tunnel().await })
    );

    match res {
        Ok(_) => Ok(()),
        Err(e) => failure::bail!(e)
    }
}

async fn start_argo_tunnel() -> Result<(), failure::Error> {
    // maybe we want to put a retry loop over this instead of using a clumsy wait.
    let tool_name = PathBuf::from("cloudflared");
    // todo: Finally get cloudflared release binaries distributed on GitHub so we could simply uncomment
    // the line below.
    // let binary_path = install::install(tool_name, "cloudflare")?.binary(tool_name)?;
    let args = ["tunnel", "--metrics", "localhost:8081"];

    let command = command(&args, &tool_name);
    let command_name = format!("{:?}", command);

    message::working("Starting up an Argo Tunnel");
    commands::run(command, &command_name)
}

async fn start_log_collection_http_server() -> Result<(), Box<dyn std::error::Error + Send + Sync>>
{
    // Start HTTP echo server that prints whatever is posted to it.
    let addr = ([127, 0, 0, 1], 8080).into();

    message::working("HTTP Echo server is running on 127.0.0.1:8080");

    let service = make_service_fn(|_| async { Ok::<_, hyper::Error>(service_fn(print_logs)) });

    let server = Server::bind(&addr).serve(service);

    server.await?;

    Ok(())
}

async fn print_logs(req: Request<Body>) -> Result<Response<Body>, hyper::Error> {
    match (req.method(), req.uri().path()) {
        (&Method::POST, "/") => {
            let whole_body = hyper::body::to_bytes(req.into_body()).await?;
            println!("{:?}", str::from_utf8(&whole_body).unwrap());

            Ok(Response::new(Body::from("Success")))
        }
        _ => {
            let mut not_found = Response::default();
            *not_found.status_mut() = StatusCode::NOT_FOUND;
            Ok(not_found)
        }
    }
}

async fn enable_tailing_start_heartbeat(
    target: &Target,
    user: &GlobalUser,
) -> Result<(), failure::Error> {
    let client = http::cf_v4_api_client_async(user, HttpApiClientConfig::default())?;

    let url = get_tunnel_url().await?;

    let response = client.request(&CreateTail {
        account_identifier: &target.account_id,
        script_name: &target.name,
        params: CreateTailParams {
            url, // how to pass URL here? Likely via a channel...
        },
    }).await;

    println!("MADE IT HERE");

    match response {
        Ok(success) => {
            let tail_id = success.result.id;

            println!("tail id is {:?}", tail_id);
            // Loop indefinitely to send "heartbeat"

            loop {
                thread::sleep(Duration::from_secs(60));
                let heartbeat_result = send_heartbeat(target, user, &client, &tail_id).await;
                if heartbeat_result.is_err() {
                    return heartbeat_result;
                }
                // This should loop forever until SIGINT is issued or Wrangler process is killed
                // through other means.
            }
        }
        Err(e) => {
            println!("ERROR {:?}", e);
            failure::bail!(http::format_error(e, None));
        }
    }

    // Ok(())
}

async fn get_tunnel_url() -> Result<String, failure::Error> {
    // todo: replace with exponential backoff retry loop until /metrics endpoint does not return 404.
    thread::sleep(Duration::from_secs(5));

    let re = Regex::new("userHostname=\"(https://[a-z.-]+)\"").unwrap();

    let body = reqwest::get("http://localhost:8081/metrics")
    .await?
    .text()
    .await?;

    for cap in re.captures_iter(&body) {
        println!("body = {}", &cap[1]);
        return Ok(cap[1].to_string())
    }
    failure::bail!("Could not extract tunnel url from cloudflared")
}

async fn send_heartbeat(
    target: &Target,
    user: &GlobalUser,
    client: &async_api::Client,
    tail_id: &str,
) -> Result<(), failure::Error> {
    let response = client.request(&CreateTailHeartbeat {
        account_identifier: &target.account_id,
        script_name: &target.name,
        tail_id: tail_id,
    }).await;

    match response {
        Ok(_) => Ok(()),
        Err(e) => failure::bail!(http::format_error(e, None)),
    }
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
