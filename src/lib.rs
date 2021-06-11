#![cfg_attr(feature = "strict", deny(warnings))]

#[macro_use]
extern crate text_io;

use cloudflare::framework::response::ApiErrors;

mod build;
pub mod cli;
pub mod preview;
pub use build::build_target;
pub mod commands;
pub mod deploy;
pub mod http;
pub mod install;
pub mod installer;
pub mod kv;
pub mod login;
pub mod reporter;
pub mod settings;
pub mod sites;
pub mod tail;
pub mod terminal;
pub mod upload;
pub mod version;
pub mod watch;
pub mod wranglerjs;

pub mod fixtures;

const TEMP_NOTICE_ES_MODULES_DO_BETA: &str = "Your account does not have permission to do this! While Durable Objects are in Beta, the modules format is limited to accounts which have opted-in to the Beta. You may do so by following the instructions here: https://developers.cloudflare.com/workers/learning/using-durable-objects";

/// Return a formatted error message from the API if present
pub fn format_api_errors(raw: String) -> Option<String> {
    if let Ok(api_errors) = serde_json::from_str::<ApiErrors>(&raw) {
        let mut err = "Something went wrong with the request to Cloudflare...\n".to_string();
        // handle possible case of opt-in required modules usage
        if api_errors
            .errors
            .iter()
            .any(|e| e.message.contains("workers.api.error.not_entitled"))
        {
            err.push_str("\n\n");
            err.push_str(TEMP_NOTICE_ES_MODULES_DO_BETA)
        }

        // add all api errors to the accumulator string
        err.push_str(
            &api_errors
                .errors
                .iter()
                .map(|e| format!("{} [API code: {}]", e.message.clone(), e.code))
                .collect::<Vec<String>>()
                .join("\n"),
        );

        return Some(err);
    }

    None
}
