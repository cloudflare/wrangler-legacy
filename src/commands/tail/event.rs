use chrono::{Local, TimeZone};
use console::style;
use serde::{Deserialize, Serialize};
use std::fmt::{Display, Formatter, Result};

/// A unique protocol ID that is passed by the `Sec-WebSocket-Protocol` header.
///
/// It is important that this header is provided, so we can safely modify
/// the protocol schema in the future without breaking clients.
pub const PROTOCOL_ID: &str = "trace-v1";

/// A trace event.
///
/// This event is fired by the Workers runtime after another event has completed.
/// Not every field is shown here, only the ones necessary for Display.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TraceEvent {
    #[serde(alias = "eventTimestamp")]
    pub timestamp: i64,
    pub outcome: String,
    pub logs: Vec<LogItem>,
    #[serde(alias = "exceptions")]
    pub errors: Vec<ErrorItem>,
    pub event: EventItem,
}

/// An event item.
///
/// * If `request` is present, it's an fetch event.
/// * If `cron` is present, it's a scheduled event.
/// * Otherwise, the event type is unknown.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventItem {
    pub request: Option<RequestItem>,
    pub cron: Option<String>,
}

/// A request item.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RequestItem {
    pub url: String,
    pub method: String,
    pub cf: Option<CfMetadata>,
}

/// Cloudflare metadata about an event.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CfMetadata {
    pub colo: String,
}

/// A log item.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogItem {
    pub level: String,
    pub message: serde_json::Value,
}

/// An error item.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorItem {
    pub name: String,
    pub message: String,
    // TODO: we really need to implement stacktraces!
}

impl Display for TraceEvent {
    fn fmt(&self, f: &mut Formatter) -> Result {
        let request = self.event.request.clone().unwrap();
        write!(
            f,
            "[{}] [{}] {} {}",
            Local
                .timestamp_millis(self.timestamp)
                .format("%Y-%m-%d %H:%M:%S"),
            match self.outcome.as_ref() {
                "ok" => style("Ok").green(),
                "canceled" => style("Canceled").blue(),
                "exception" => style("Error").red(),
                "exceededCpu" => style("Exceeded CPU").yellow(),
                _ => style("System Error").red(),
            }
            .bold(),
            request.method,
            style(request.url).bold()
        )?;
        for log in self.logs.iter() {
            write!(f, "\n |> {}", log)?;
        }
        for err in self.errors.iter() {
            write!(f, "\n |! {}", err)?;
        }
        Ok(())
    }
}

impl Display for LogItem {
    fn fmt(&self, f: &mut Formatter) -> Result {
        write!(
            f,
            "[{}] {}",
            match self.level.as_ref() {
                "debug" => style("Debug"),
                "warn" => style("Warn").yellow(),
                "error" => style("Error").red(),
                _ => style("Info").blue(),
            }
            .bold(),
            self.message
        )
    }
}

impl Display for ErrorItem {
    fn fmt(&self, f: &mut Formatter) -> Result {
        write!(
            f,
            "[{}] {}",
            style(&self.name).red().bold(),
            style(&self.message).bold()
        )
    }
}
