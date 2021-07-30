use chrono::{Local, TimeZone};
use console::style;
use serde::{Deserialize, Serialize};
use serde_json::Value;
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
    pub exceptions: Vec<ExceptionItem>,
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
    pub message: Value,
}

/// An error item.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExceptionItem {
    pub name: String,
    pub message: String,
    // TODO(soon): we really need to implement stacktraces.
}

impl Display for TraceEvent {
    fn fmt(&self, f: &mut Formatter) -> Result {
        let timestamp = style(
            Local
                .timestamp_millis(self.timestamp)
                .format("%Y-%m-%d %H:%M:%S"),
        )
        .dim();
        let outcome = match self.outcome.as_ref() {
            "ok" => style("Ok").green(),
            "canceled" => style("Canceled").yellow(),
            "exception" => style("Error").red(),
            "exceededCpu" => style("Exceeded Limit").red(),
            _ => style("System Error").red(),
        };
        match self.event.request.clone() {
            Some(request) => {
                let colo = style(
                    request
                        .cf
                        .map(|cf| cf.colo)
                        .unwrap_or_else(|| "?".to_owned()),
                )
                .dim();
                let method = style(request.method);
                let url = style(request.url).bold();
                write!(
                    f,
                    "[{}] [{}] [{}] {} {}",
                    timestamp, colo, outcome, method, url
                )
            }
            _ => match self.event.cron.clone() {
                // TODO(soon): add colo to scheduled event.
                Some(cron) => write!(f, "[{}] [?] [{}] {}", timestamp, outcome, cron),
                _ => write!(f, "[{}] [?] [{}] <unknown event>", timestamp, outcome),
            },
        }?;
        for log in self.logs.iter() {
            let prefix = style("|").dim();
            write!(f, "\n {} {}", prefix, log)?;
        }
        for err in self.exceptions.iter() {
            let prefix = style("!").dim();
            write!(f, "\n {} {}", prefix, err)?;
        }
        Ok(())
    }
}

impl Display for LogItem {
    fn fmt(&self, f: &mut Formatter) -> Result {
        let level = match self.level.as_ref() {
            "debug" => style("Debug").blue(),
            "warn" => style("Warn").yellow(),
            "error" => style("Error").red(),
            _ => style("Info").dim(),
        };
        write!(f, "[{}] ", level)?;
        match &self.message {
            // Most console.log() messages are formatted as an array.
            // e.g.
            //   console.log('Hi')             // => '["Hi"]'
            //   console.log('Hello', 'World') // => '["Hello","World"]'
            //
            // However, we want to format it nicely, similar to how it's done in DevTools.
            // e.g.
            //   Hello World
            //
            // While a recursive approach might seem like a good idea, the output becomes
            // suprisingly unreadable. Instead, we only handle the simple case where the
            // top-level is an array and its values are strings.
            Value::Array(values) => {
                for value in values.iter() {
                    match value {
                        Value::String(s) => write!(f, "{}", s),
                        v => write!(f, "{}", v),
                    }?;
                    write!(f, " ")?;
                }
                Ok(())
            }
            Value::String(v) => write!(f, "{}", v),
            v => write!(f, "{}", v),
        }?;
        Ok(())
    }
}

impl Display for ExceptionItem {
    fn fmt(&self, f: &mut Formatter) -> Result {
        let name = style(&self.name).red().bold();
        let message = style(&self.message).red();
        write!(f, "[{}] {}", name, message)
    }
}
