use std::str;
use std::time::Duration;

use regex::Regex;
use reqwest;
use tokio::sync::oneshot::{Receiver, Sender};
use tokio::time::{delay_for, Delay};

use cloudflare::endpoints::workers::{CreateTail, CreateTailParams, SendTailHeartbeat};
use cloudflare::framework::HttpApiClientConfig;
use cloudflare::framework::{async_api, async_api::ApiClient};

use crate::http;
use crate::settings::global_user::GlobalUser;
use crate::settings::toml::Target;

const KEEP_ALIVE_INTERVAL: u64 = 60;

pub struct Session;

/// Session is responsible for interacting with the Workers API to establish and maintain the tail
/// connection; once the tail session is established, it uses the returned tail_id to send periodic
/// "keepalive" heartbeats to the service. If the service does not receive a heartbeat for ~10
/// minutes, it will kill the tail session by removing the trace worker.
impl Session {
    pub async fn run(
        target: Target,
        user: GlobalUser,
        shutdown_rx: Receiver<()>,
        tx: Sender<()>,
    ) -> Result<(), failure::Error> {
        // We need to exit on a shutdown command without waiting for API calls to complete.
        tokio::select! {
            _ = shutdown_rx => { Ok(()) }
            result = Session::start(target, user, tx) => { result }
        }
    }

    async fn start(target: Target, user: GlobalUser, tx: Sender<()>) -> Result<(), failure::Error> {
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
                let mut delay = delay_for(duration);

                loop {
                    delay.await;
                    let heartbeat_result = send_heartbeat(&target, &client, &tail_id).await;
                    if heartbeat_result.is_err() {
                        return heartbeat_result;
                    }
                    delay = delay_for(duration);
                }
            }
            Err(e) => {
                tx.send(()).unwrap();
                failure::bail!(http::format_error(e, Some(&tail_help)))
            }
        }
    }
}

async fn get_tunnel_url() -> Result<String, failure::Error> {
    // regex for extracting url from cloudflared metrics port.
    let url_regex = Regex::new("userHostname=\"(https://[a-z.-]+)\"").unwrap();

    struct RetryDelay {
        delay: Delay,
        attempt: u64,
        max_attempts: u64,
    }

    // This retry loop retries retrieving the cloudflared endpoint url from the cloudflared /metrics
    // until it gets the URL or has tried retrieving the URL over 5 times.
    impl RetryDelay {
        fn new(max_attempts: u64) -> RetryDelay {
            RetryDelay {
                delay: delay_for(Duration::from_millis(0)),
                attempt: 0,
                max_attempts,
            }
        }

        // our retry delay is an [exponential backoff](https://en.wikipedia.org/wiki/Exponential_backoff),
        // which simply waits twice as long between each attempt to avoid hammering the LogServer.
        fn reset(self) -> RetryDelay {
            let attempt = self.attempt + 1;
            let delay = delay_for(Duration::from_millis(attempt * attempt * 1000));
            let max_attempts = self.max_attempts;

            RetryDelay {
                attempt,
                delay,
                max_attempts,
            }
        }

        fn expired(&self) -> bool {
            self.attempt > self.max_attempts
        }

        fn is_elapsed(&self) -> bool {
            self.delay.is_elapsed()
        }
    }

    let mut delay = RetryDelay::new(5);

    while !delay.expired() {
        if delay.is_elapsed() {
            if let Ok(resp) = reqwest::get("http://localhost:8081/metrics").await {
                let body = resp.text().await?;

                for url_match in url_regex.captures_iter(&body) {
                    let url = url_match[1].to_string();
                    return Ok(url);
                }
            }

            delay = delay.reset();
        }
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
        Err(e) => failure::bail!(http::format_error(e, Some(&tail_help))),
    }
}

// tail_help() provides more detailed explanations of Workers KV API error codes.
// See https://api.cloudflare.com/#workers-kv-namespace-errors for details.
fn tail_help(error_code: u16) -> &'static str {
    match error_code {
        7003 | 7000 => {
            "Your wrangler.toml is likely missing the field \"account_id\", which is required to tail a worker."
        }
        // unauthorized
        10000 => {
            "Make sure your API token has permission to both edit and read workers on your account"
        }
        // script not found
        10007 => "wrangler can only tail live Worker scripts. Run `wrangler publish` before attempting to tail.", // key errors
        // limit errors
        10057 | 10058 | 10059 => "See documentation",
        _ => "",
    }
}
