use std::str;
use std::thread;
use std::time::Duration;

use regex::Regex;
use reqwest;
use tokio::sync::oneshot::error::TryRecvError;
use tokio::sync::oneshot::Receiver;
use tokio::time;

use cloudflare::endpoints::workers::{CreateTail, CreateTailParams, SendTailHeartbeat};
use cloudflare::framework::HttpApiClientConfig;
use cloudflare::framework::{async_api, async_api::ApiClient};

use crate::http;
use crate::settings::global_user::GlobalUser;
use crate::settings::toml::Target;

const KEEP_ALIVE_INTERVAL: u64 = 60;

pub struct Session;

impl Session {
    pub async fn run(
        target: Target,
        user: GlobalUser,
        mut rx: Receiver<()>,
    ) -> Result<(), failure::Error> {
        let client = http::cf_v4_api_client_async(&user, HttpApiClientConfig::default())?;

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
                // This should loop forever until SIGINT is issued or Wrangler process is killed
                // through other means.
                let duration = Duration::from_millis(1000 * KEEP_ALIVE_INTERVAL);
                let mut delay = time::delay_for(duration);

                loop {
                    match rx.try_recv() {
                        Err(TryRecvError::Empty) => {
                            if delay.is_elapsed() {
                                let heartbeat_result =
                                    send_heartbeat(&target, &client, &tail_id).await;
                                if heartbeat_result.is_err() {
                                    return heartbeat_result;
                                }
                                delay = time::delay_for(duration);
                            }
                        }
                        _ => {
                            return Ok(());
                        }
                    }
                }
            }
            Err(e) => failure::bail!(http::format_error(e, None)),
        }
    }
}

async fn get_tunnel_url() -> Result<String, failure::Error> {
    // regex for extracting url from cloudflared metrics port.
    let url_regex = Regex::new("userHostname=\"(https://[a-z.-]+)\"").unwrap();

    let mut attempt = 0;

    // This exponential backoff retry loop retries retrieving the cloudflared endpoint url
    // from the cloudflared /metrics endpoint until it gets the URL or has tried retrieving the URL
    // over 5 times.
    while attempt < 5 {
        if let Ok(resp) = reqwest::get("http://localhost:8081/metrics").await {
            let body = resp.text().await?;

            for cap in url_regex.captures_iter(&body) {
                return Ok(cap[1].to_string());
            }
        }

        attempt += 1;
        thread::sleep(Duration::from_millis(attempt * attempt * 1000));
    }

    failure::bail!("Could not extract tunnel url from cloudflared")
}

async fn send_heartbeat(
    target: &Target,
    client: &async_api::Client,
    tail_id: &str,
) -> Result<(), failure::Error> {
    let response = client
        .request(&SendTailHeartbeat {
            account_identifier: &target.account_id,
            script_name: &target.name,
            tail_id,
        })
        .await;

    match response {
        Ok(_) => Ok(()),
        Err(e) => failure::bail!(http::format_error(e, None)),
    }
}
