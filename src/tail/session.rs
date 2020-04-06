use std::str;
use std::thread;
use std::time::Duration;

use regex::Regex;
use reqwest;

use cloudflare::endpoints::workers::{CreateTail, CreateTailParams, SendTailHeartbeat};
use cloudflare::framework::HttpApiClientConfig;
use cloudflare::framework::{async_api, async_api::ApiClient};

use crate::http;
use crate::settings::global_user::GlobalUser;
use crate::settings::toml::Target;

pub struct Session;

impl Session {
    pub async fn run(target: &Target, user: &GlobalUser) -> Result<(), failure::Error> {
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
