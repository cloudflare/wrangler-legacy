#![cfg_attr(feature = "strict", deny(warnings))]
#![warn(clippy::todo)] // TODO(jyn514): remove this once clippy warns about it by default

#[macro_use]
extern crate text_io;

#[macro_use]
extern crate erased_serde;

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
pub mod terminal;
pub mod upload;
pub mod version;
pub mod watch;
pub mod wranglerjs;

const TEMP_NOTICE_ES_MODULES_DO_BETA: &str = "Your account does not have permission to do this! While Durable Objects are in Beta, the modules format is limited to accounts which have opted-in to the Beta. You may do so by following the instructions here: https://developers.cloudflare.com/workers/learning/using-durable-objects";

/// Return a formatted error message from the API if present, or raw value if not
pub fn format_api_errors(raw: String) -> String {
    let mut msg = "Something went wrong with the request to Cloudflare...\n".to_string();
    if let Ok(api_errors) = serde_json::from_str::<ApiErrors>(&raw) {
        // handle possible case of opt-in required modules usage
        // TODO: remove this after DO beta restrictions are lifted
        if api_errors
            .errors
            .iter()
            .any(|e| e.message.contains("workers.api.error.not_entitled"))
        {
            msg.push_str("\n\n");
            msg.push_str(TEMP_NOTICE_ES_MODULES_DO_BETA)
        }

        // add all api errors to the accumulator string
        let formatted_errors: Vec<String> = api_errors
            .errors
            .iter()
            .map(|e| format!("{} [API code: {}]", e.message.clone(), e.code))
            .collect();
        msg.push_str(&formatted_errors.join("\n"));

        return msg;
    }

    // if we have no useful detail to extract from the API error, it is likely better to print the
    // raw response value so the end-user can attempt to resolve an issue
    msg.push('\n');
    msg.push_str(&raw);
    msg
}
