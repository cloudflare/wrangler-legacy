use std::str;
use std::time::Duration;

use tokio::sync::oneshot::{Receiver, Sender};
use tokio::time::delay_for;

use cloudflare::endpoints::workers::{CreateTail, CreateTailParams, DeleteTail, SendTailHeartbeat};
use cloudflare::framework::HttpApiClientConfig;
use cloudflare::framework::{async_api, async_api::ApiClient};

use crate::http;
use crate::settings::global_user::GlobalUser;
use crate::settings::toml::Target;

const KEEP_ALIVE_DURATION: Duration = Duration::from_millis(60_000);

pub struct Session {
    tail_id: Option<String>,
}

/// Session is responsible for interacting with the Workers API to establish and maintain the tail
/// connection; once the tail session is established, it uses the returned tail_id to send periodic
/// "keepalive" heartbeats to the service. If the service does not receive a heartbeat for ~10
/// minutes, it will kill the tail session by removing the trace worker.
impl Session {
    pub async fn run(
        target: Target,
        user: GlobalUser,
        tail_url: String,
        shutdown_rx: Receiver<()>,
        tx: Sender<()>,
    ) -> Result<(), failure::Error> {
        // During the start process we'll populate the tail with the response from the API.
        let mut session = Session { tail_id: None };
        // We need to exit on a shutdown command without waiting for API calls to complete.
        tokio::select! {
            _ = shutdown_rx => { session.close(&user, &target).await }
            result = session.start(&target, &user, &tail_url, tx) => { result }
        }
    }

    async fn close(&self, user: &GlobalUser, target: &Target) -> Result<(), failure::Error> {
        // The API will clean up tails after about 10 minutes of inactivity, or 24 hours of
        // activity but since we limit the number of tails allowed on a single script we should at
        // least try to delete them as we go.
        if let Some(tail_id) = &self.tail_id {
            let client = http::cf_v4_api_client_async(&user, HttpApiClientConfig::default())?;
            client
                .request(&DeleteTail {
                    account_identifier: &target.account_id,
                    script_name: &target.name,
                    tail_id: &tail_id,
                })
                .await?;
        }

        Ok(())
    }

    async fn start(
        &mut self,
        target: &Target,
        user: &GlobalUser,
        tail_url: &str,
        tx: Sender<()>,
    ) -> Result<(), failure::Error> {
        let client = http::cf_v4_api_client_async(user, HttpApiClientConfig::default())?;
        let response = client
            .request(&CreateTail {
                account_identifier: &target.account_id,
                script_name: &target.name,
                params: CreateTailParams {
                    url: tail_url.to_string(),
                },
            })
            .await;

        match response {
            Ok(success) => {
                let tail_id = success.result.id;
                self.tail_id = Some(tail_id.clone());

                // Loop indefinitely to send "heartbeat" to API and keep log streaming alive.
                // This should loop forever until SIGINT is issued or wrangler process is killed.
                let mut delay = delay_for(KEEP_ALIVE_DURATION);

                loop {
                    delay.await;
                    let heartbeat_result = send_heartbeat(&target, &client, &tail_id).await;
                    if heartbeat_result.is_err() {
                        return heartbeat_result;
                    }
                    delay = delay_for(KEEP_ALIVE_DURATION);
                }
            }
            Err(e) => {
                tx.send(()).unwrap();
                failure::bail!(http::format_error(e, Some(&tail_help)))
            }
        }
    }
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

// Provides helpful hints when encountering errors from the Cloudflare API.
fn tail_help(error_code: u16) -> &'static str {
    match error_code {
        7003 | 7000 => "Did you define your `account_id` in the `wrangler.toml`?",
        10000 => "Does your API token have permission to read and write Worker scripts?",
        10007 => "Did you run `wrangler publish` yet? You can only tail a published Worker.",
        10057 => "Sorry! Your Worker has too much traffic to view live logs.",
        10058 => "Have you run `wrangler tail` multiple times? You have too many active tails.",
        _ => "",
    }
}
