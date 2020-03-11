use crate::http;
use crate::settings::global_user::GlobalUser;
use crate::settings::toml::Target;

use cloudflare::endpoints::workers::{CreateTail, CreateTailHeartbeat, CreateTailParams};
use cloudflare::framework::HttpApiClientConfig;
use cloudflare::framework::{async_api, async_api::ApiClient};

use std::path::PathBuf;
use std::process::Stdio;
use std::str;
use std::thread;
use std::time::Duration;

use hyper::service::{make_service_fn, service_fn};
use hyper::{Body, Method, Request, Response, Server, StatusCode};
use log::log_enabled;
use log::Level::Info;
use regex::Regex;
use reqwest;
use tokio;
use tokio::process::Command;
use tokio::runtime::Runtime as TokioRuntime;

pub fn start_tail(target: &Target, user: &GlobalUser) -> Result<(), failure::Error> {
    let mut runtime = TokioRuntime::new()?;
    // Note that we use eprintln!() throughout this file; this is because we want any
    // helpful output to not be mixed with actual log JSON output, so we use this macro
    // to print messages to stderr instead of stdout (where log output is printed).
    eprintln!(
        "Setting up log streaming from Worker \"{}\" to Wrangler. This may take a few seconds...",
        target.name
    );

    // todo: get rid of this gross clone. Maybe just default to static lifetime.
    runtime.block_on(run_cloudflared_start_server(target.clone(), user.clone()))
}

async fn run_cloudflared_start_server(
    target: Target,
    user: GlobalUser,
) -> Result<(), failure::Error> {
    let res = tokio::try_join!(
        start_log_collection_http_server(),
        start_argo_tunnel(),
        enable_tailing_start_heartbeat(&target, &user)
    );

    match res {
        Ok(_) => Ok(()),
        Err(e) => failure::bail!(e),
    }
}

async fn start_argo_tunnel() -> Result<(), failure::Error> {
    // todo:remove sleep!! Can maybe use channel to signal from http server thread to argo tunnel
    // thread that the server is ready on port 8080 and prepared for the cloudflared CLI to open an
    // Argo Tunnel to it.
    thread::sleep(Duration::from_secs(5));

    let tool_name = PathBuf::from("cloudflared");
    // todo: Finally get cloudflared release binaries distributed on GitHub so we could simply uncomment
    // the line below.
    // let binary_path = install::install(tool_name, "cloudflare")?.binary(tool_name)?;

    // todo: allow user to pass in their own ports in case ports 8080 (used by cloudflared)
    // and 8081 (used by cloudflared metrics) are both already being used.
    let args = ["tunnel", "--metrics", "localhost:8081"];

    let mut command = command(&args, &tool_name);
    let command_name = format!("{:?}", command);

    let status = command
        .kill_on_drop(true)
        .spawn()
        .expect(&format!("{} failed to spawn", command_name))
        .await?;

    if !status.success() {
        failure::bail!(
            "tried running command:\n{}\nexited with {}",
            command_name.replace("\"", ""),
            status
        )
    }
    Ok(())
}

async fn start_log_collection_http_server() -> Result<(), failure::Error> {
    // Start HTTP echo server that prints whatever is posted to it.
    let addr = ([127, 0, 0, 1], 8080).into();

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

    let response = client
        .request(&CreateTail {
            account_identifier: &target.account_id,
            script_name: &target.name,
            params: CreateTailParams { url },
        })
        .await;

    match response {
        Ok(success) => {
            eprintln!("Now prepared to stream logs.");

            let tail_id = success.result.id;

            // Loop indefinitely to send "heartbeat" to API and keep log streaming alive.
            loop {
                thread::sleep(Duration::from_secs(60));
                let heartbeat_result = send_heartbeat(target, &client, &tail_id).await;
                if heartbeat_result.is_err() {
                    return heartbeat_result;
                }
                // This should loop forever until SIGINT is issued or Wrangler process is killed
                // through other means.
            }
        }
        Err(e) => failure::bail!(http::format_error(e, None)),
    }
}

async fn get_tunnel_url() -> Result<String, failure::Error> {
    // regex for extracting url from cloudflared metrics port.
    let re = Regex::new("userHostname=\"(https://[a-z.-]+)\"").unwrap();

    let mut attempt = 0;

    // This exponential backoff retry loop retries retrieving the cloudflared endpoint url
    // from the cloudflared /metrics endpoint until it gets the URL or has tried retrieving the URL
    // over 5 times.
    while attempt < 5 {
        if let Ok(resp) = reqwest::get("http://localhost:8081/metrics").await {
            let body = resp.text().await?;

            for cap in re.captures_iter(&body) {
                return Ok(cap[1].to_string());
            }
        }

        attempt = attempt + 1;
        thread::sleep(Duration::from_millis(attempt * attempt * 100));
    }

    failure::bail!("Could not extract tunnel url from cloudflared")
}

async fn send_heartbeat(
    target: &Target,
    client: &async_api::Client,
    tail_id: &str,
) -> Result<(), failure::Error> {
    let response = client
        .request(&CreateTailHeartbeat {
            account_identifier: &target.account_id,
            script_name: &target.name,
            tail_id: tail_id,
        })
        .await;

    match response {
        Ok(_) => Ok(()),
        Err(e) => failure::bail!(http::format_error(e, None)),
    }
}

// todo: let's not clumsily copy this from commands/build/mod.rs
// We definitely want to keep the check for RUST_LOG=info below so we avoid
// spamming user terminal with default cloudflared output (which is pretty darn sizable.)
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
    // Let user read cloudflared process logs iff RUST_LOG=info.
    if !log_enabled!(Info) {
        c.stderr(Stdio::null());
    }

    c
}
