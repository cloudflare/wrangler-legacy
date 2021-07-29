use serde::Serialize;
use std::collections::HashSet;
use std::convert::From;

/// A filter that accepts trace events.
///
/// wrangler does not handle any of the filtering,
/// it only uploads them to the `WebSocketTail`.
pub trait TraceFilter: erased_serde::Serialize {}

#[derive(Debug, Clone, Serialize)]
pub struct OutcomeFilter {
    pub outcome: Vec<String>,
}

impl TraceFilter for OutcomeFilter {}

impl From<Vec<String>> for OutcomeFilter {
    fn from(outcomes: Vec<String>) -> Self {
        let mut results = HashSet::new();
        for outcome in outcomes {
            match outcome.as_ref() {
                "ok" => results.insert("ok"),
                "canceled" => results.insert("canceled"),
                "error" => {
                    results.insert("exception");
                    results.insert("exceededCpu");
                    results.insert("unknown")
                }
                _ => false,
            };
        }
        Self {
            outcome: results.into_iter().map(String::from).collect(),
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct SamplingRateFilter {
    pub sampling_rate: f64,
}

impl TraceFilter for SamplingRateFilter {}

impl From<f64> for SamplingRateFilter {
    fn from(sampling_rate: f64) -> Self {
        Self { sampling_rate }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct MethodFilter {
    pub method: Vec<String>,
}

impl TraceFilter for MethodFilter {}

impl From<Vec<String>> for MethodFilter {
    fn from(method: Vec<String>) -> Self {
        Self { method }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct HeaderFilter {
    pub key: String,
    pub query: Option<String>,
}

impl TraceFilter for HeaderFilter {}

impl From<String> for HeaderFilter {
    fn from(header: String) -> Self {
        match header.split_once(":") {
            None => Self {
                key: header,
                query: None,
            },
            Some((key, value)) => Self {
                key: key.trim_end().to_owned(),
                query: Some(value.trim_start().to_owned()),
            },
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct ClientIpFilter {
    pub client_ip: Vec<String>,
}

impl TraceFilter for ClientIpFilter {}

impl From<Vec<String>> for ClientIpFilter {
    fn from(client_ip: Vec<String>) -> Self {
        Self { client_ip }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct QueryFilter {
    pub query: String,
}

impl TraceFilter for QueryFilter {}

impl From<String> for QueryFilter {
    fn from(query: String) -> Self {
        Self { query }
    }
}

// By default, serde::Serialize does not handle embeded traits, this fixes that.
serialize_trait_object!(TraceFilter);
