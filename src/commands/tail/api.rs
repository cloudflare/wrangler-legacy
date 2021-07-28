use crate::http;
use crate::settings::global_user::GlobalUser;

use anyhow::Result;
use chrono::{DateTime, Utc};
use cloudflare::{
    endpoints::workers::{CreateTail, CreateTailParams, DeleteTail, SendTailHeartbeat},
    framework::{async_api::ApiClient, response::ApiFailure},
};
use reqwest::StatusCode;
use std::time::{SystemTime, UNIX_EPOCH};
use tokio::time::{Duration, Instant};
use url::Url;

/// A tail captures `TraceEvent`s from a published Worker.
#[derive(Debug, Clone)]
pub struct Tail {
    pub user: GlobalUser,
    pub account_id: String,
    pub script_name: String,
    pub expires_at: Instant,
    pub url: Option<Url>,
    pub id: Option<String>,
}

impl Tail {
    /// Sets up a new tail, but does not actually create it.
    pub fn new(
        user: GlobalUser,
        account_id: String,
        script_name: String,
        url: Option<Url>,
    ) -> Self {
        Self {
            user,
            account_id,
            script_name,
            expires_at: Instant::now(),
            url,
            id: None,
        }
    }

    /// Tests if the tail is using WebSockets.
    pub fn is_web_socket(&self) -> bool {
        if let Some(url) = self.url.clone() {
            return matches!(url.scheme(), "ws" | "wss");
        }
        false
    }

    /// Creates the tail and attaches it to a Worker.
    ///
    /// If successful, the tail must be kept-alive before its expiration time.
    pub async fn create(&mut self) -> Result<()> {
        match self.id {
            None => match http::cf_v4_api_client_async(&self.user)?
                .request(&CreateTail {
                    account_identifier: &self.account_id,
                    script_name: &self.script_name,
                    params: CreateTailParams {
                        url: self.url.clone().map(String::from),
                    },
                })
                .await
            {
                Ok(response) => {
                    let tail = response.result;
                    log::info!("Created tail: {:?}", tail);
                    self.id = Some(tail.id);
                    self.expires_at = to_instant(tail.expires_at);
                    self.url = Some(Url::parse(
                        &tail.url.expect("Expected a URL from tail response"),
                    )?);
                    Ok(())
                }
                Err(err) => {
                    anyhow::bail!("Failed to create tail: {}", http::format_error(err, None))
                }
            },
            _ => Ok(()),
        }
    }

    /// Sends a keep-alive to the tail.
    pub async fn keep_alive(&mut self) -> Result<()> {
        match self.id.clone() {
            Some(tail_id) => {
                match http::cf_v4_api_client_async(&self.user)?
                    .request(&SendTailHeartbeat {
                        account_identifier: &self.account_id,
                        script_name: &self.script_name,
                        tail_id: &tail_id,
                    })
                    .await
                {
                    Ok(response) => {
                        log::debug!("Sent tail keep-alive tail: {:?}", response.result);
                        self.expires_at = to_instant(response.result.expires_at);
                        Ok(())
                    }
                    Err(err) => {
                        anyhow::bail!(
                            "Failed to keep-alive tail: {}",
                            http::format_error(err, None)
                        )
                    }
                }
            }
            _ => Ok(()),
        }
    }

    /// Deletes the tail and unattaches it from the Worker.
    pub async fn delete(&mut self) -> Result<()> {
        match self.id.clone() {
            Some(tail_id) => match http::cf_v4_api_client_async(&self.user)?
                .request(&DeleteTail {
                    account_identifier: &self.account_id,
                    script_name: &self.script_name,
                    tail_id: &tail_id,
                })
                .await
            {
                Ok(_) | Err(ApiFailure::Error(StatusCode::NOT_FOUND, _)) => {
                    log::info!("Deleted tail: {}", &tail_id);
                    self.id = None;
                    self.url = None;
                    self.expires_at = Instant::now();
                    Ok(())
                }
                Err(err) => {
                    anyhow::bail!("Failed to delete tail: {}", http::format_error(err, None))
                }
            },
            _ => Ok(()),
        }
    }
}

/// Converts a `chrono::DateTime` into a `tokio::time::Instant`.
fn to_instant(datetime: DateTime<Utc>) -> Instant {
    let delta = datetime.timestamp()
        - SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("Time is going backwards?")
            .as_secs() as i64;
    Instant::now() + Duration::from_secs(delta as u64)
}
