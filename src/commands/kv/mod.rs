use cloudflare::framework::auth::Credentials;
use cloudflare::framework::response::ApiFailure;
use cloudflare::framework::HttpApiClient;
use http::status::StatusCode;
use percent_encoding::{percent_encode, PATH_SEGMENT_ENCODE_SET};

use crate::settings::global_user::GlobalUser;
use crate::terminal::message;

pub mod bulk;
pub mod key;
pub mod namespace;

// Truncate all "yes", "no" responses for interactive delete prompt to just "y" or "n".
const INTERACTIVE_RESPONSE_LEN: usize = 1;
const YES: &str = "y";
const NO: &str = "n";

fn api_client(user: GlobalUser) -> Result<HttpApiClient, failure::Error> {
    Ok(HttpApiClient::new(Credentials::from(user)))
}

fn print_error(e: ApiFailure) {
    match e {
        ApiFailure::Error(status, api_errors) => {
            give_status_code_context(status);
            for error in api_errors.errors {
                message::warn(&format!("Error {}: {}", error.code, error.message));

                let suggestion = help(error.code);
                if !suggestion.is_empty() {
                    message::help(suggestion);
                }
            }
        }
        ApiFailure::Invalid(reqwest_err) => message::warn(&format!("Error: {}", reqwest_err)),
    }
}

// For interactively handling deletes (and discouraging accidental deletes).
// Input like "yes", "Yes", "no", "No" will be accepted, thanks to the whitespace-stripping
// and lowercasing logic below.
fn interactive_delete(prompt_string: &str) -> Result<bool, failure::Error> {
    println!("{} [y/n]", prompt_string);
    let mut response: String = read!("{}\n");
    response = response.split_whitespace().collect(); // remove whitespace
    response.make_ascii_lowercase(); // ensure response is all lowercase
    response.truncate(INTERACTIVE_RESPONSE_LEN); // at this point, all valid input will be "y" or "n"
    match response.as_ref() {
        YES => Ok(true),
        NO => Ok(false),
        _ => failure::bail!("Response must either be \"y\" for yes or \"n\" for no"),
    }
}

fn url_encode_key(key: &str) -> String {
    percent_encode(key.as_bytes(), PATH_SEGMENT_ENCODE_SET).to_string()
}

// For handling cases where the API gateway returns errors via HTTP status codes
// (no KV error code is given).
fn give_status_code_context(status_code: StatusCode) {
    match status_code {
        StatusCode::PAYLOAD_TOO_LARGE => message::warn("Returned status code 413, Payload Too Large. Make sure your upload is less than 100MB in size"),
        _ => (),
    }
}

fn help(error_code: u16) -> &'static str {
    // https://api.cloudflare.com/#workers-kv-namespace-errors
    match error_code {
        // namespace errors
        10010 | 10011 | 10012 | 10013 | 10014 | 10018 => {
            "Run `wrangler kv list` to see your existing namespaces with IDs"
        }
        10009 => "Run `wrangler kv list <namespaceID>` to see your existing keys", // key errors
        // TODO: link to more info
        // limit errors
        10022 | 10024 | 10030 => "See documentation",
        // TODO: link to tool for this?
        // legacy namespace errors
        10021 | 10035 | 10038 => "Consider moving this namespace",
        // cloudflare account errors
        10017 | 10026 => "Workers KV is a paid feature, please upgrade your account (https://www.cloudflare.com/products/workers-kv/)",
        _ => "",
    }
}
