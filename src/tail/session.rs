use std::str;
use std::time::Duration;

use regex::Regex;
use reqwest;
use tokio::sync::oneshot::error::TryRecvError;
use tokio::sync::oneshot::Receiver;
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
        mut shutdown_rx: Receiver<()>,
    ) -> Result<(), failure::Error> {
        let client = http::cf_v4_api_client_async(&user, HttpApiClientConfig::default())?;

        let url = get_tunnel_url(&mut shutdown_rx).await?;

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
                    match shutdown_rx.try_recv() {
                        // this variant of the [TryRecvError](https://docs.rs/tokio/0.2.16/tokio/sync/oneshot/error/enum.TryRecvError.html)
                        // occurs when the receiver listens for a message and the channel is empty;
                        // in this case, it means that we have not received a shutdown command and
                        // can continue with our task. The other variant would indicate that the
                        // sender has been dropped, in which case we want to follow shut down as if
                        // we received the signal.
                        Err(TryRecvError::Empty) => {
                            if delay.is_elapsed() {
                                let heartbeat_result =
                                    send_heartbeat(&target, &client, &tail_id).await;
                                if heartbeat_result.is_err() {
                                    return heartbeat_result;
                                }
                                delay = delay_for(duration);
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

async fn get_tunnel_url(shutdown_rx: &mut Receiver<()>) -> Result<String, failure::Error> {
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
        match shutdown_rx.try_recv() {
            Err(TryRecvError::Empty) => {
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
            _ => {
                return Ok("".to_string());
            }
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
        Err(e) => failure::bail!(http::format_error(e, None)),
    }
}
