use std::collections::HashSet;

use cloudflare::framework::auth::Credentials;
use cloudflare::framework::response::ApiFailure;
use cloudflare::framework::HttpApiClient;

use http::status::StatusCode;
use percent_encoding::{percent_encode, PATH_SEGMENT_ENCODE_SET};

use crate::settings::global_user::GlobalUser;
use crate::settings::target::Target;
use crate::terminal::emoji;
use crate::terminal::message;

pub mod bucket;
pub mod bulk;
pub mod key;
pub mod namespace;

// Truncate all "yes", "no" responses for interactive delete prompt to just "y" or "n".
const INTERACTIVE_RESPONSE_LEN: usize = 1;
const YES: &str = "y";
const NO: &str = "n";

fn check_duplicate_namespaces(target: &Target) -> bool {
    // HashSet for detecting duplicate namespace bindings
    let mut binding_names: HashSet<String> = HashSet::new();

    if let Some(namespaces) = &target.kv_namespaces {
        for namespace in namespaces {
            // Check if this is a duplicate binding
            if binding_names.contains(&namespace.binding) {
                return true;
            } else {
                binding_names.insert(namespace.binding.clone());
            }
        }
    }
    false
}

// Get namespace id for a given binding name.
pub fn get_namespace_id(target: &Target, binding: &str) -> Result<String, failure::Error> {
    if check_duplicate_namespaces(&target) {
        failure::bail!(
            "Namespace binding \"{}\" is duplicated in \"{}\"",
            binding,
            target.name
        )
    }

    if let Some(namespaces) = &target.kv_namespaces {
        for namespace in namespaces {
            if namespace.binding == binding {
                return Ok(namespace.id.to_string());
            }
        }
    }
    failure::bail!(
        "Namespace binding \"{}\" not found in \"{}\"",
        binding,
        target.name
    )
}

fn api_client(user: GlobalUser) -> Result<HttpApiClient, failure::Error> {
    Ok(HttpApiClient::new(Credentials::from(user)))
}

fn format_error(e: ApiFailure) -> String {
    match e {
        ApiFailure::Error(status, api_errors) => {
            give_status_code_context(status);
            let mut complete_err = "".to_string();
            for error in api_errors.errors {
                let error_msg =
                    format!("{} Error {}: {}\n", emoji::WARN, error.code, error.message);

                let suggestion = help(error.code);
                if !suggestion.is_empty() {
                    let help_msg = format!("{} {}\n", emoji::SLEUTH, suggestion);
                    complete_err.push_str(&format!("{}{}", error_msg, help_msg));
                } else {
                    complete_err.push_str(&error_msg)
                }
            }
            complete_err
        }
        ApiFailure::Invalid(reqwest_err) => format!("{} Error: {}", emoji::WARN, reqwest_err),
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
    if let StatusCode::PAYLOAD_TOO_LARGE = status_code {
        message::warn("Returned status code 413, Payload Too Large. Make sure your upload is less than 100MB in size")
    }
}

fn help(error_code: u16) -> &'static str {
    // https://api.cloudflare.com/#workers-kv-namespace-errors
    match error_code {
        // namespace errors
        10010 | 10011 | 10012 | 10013 | 10014 | 10018 => {
            "Run `wrangler kv:namespace list` to see your existing namespaces with IDs"
        }
        10009 => "Run `wrangler kv:key list` to see your existing keys", // key errors
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

#[cfg(test)]
mod tests {
    use crate::commands::kv;
    use crate::settings::target::{KvNamespace, Target, TargetType};

    #[test]
    fn it_can_detect_duplicate_bindings() {
        let target_with_dup_kv_bindings = Target {
            account_id: "".to_string(),
            kv_namespaces: Some(vec![
                KvNamespace {
                    id: "fake".to_string(),
                    binding: "KV".to_string(),
                    bucket: None,
                },
                KvNamespace {
                    id: "fake".to_string(),
                    binding: "KV".to_string(),
                    bucket: None,
                },
            ]),
            name: "test-target".to_string(),
            target_type: TargetType::Webpack,
            route: None,
            routes: None,
            webpack_config: None,
            workers_dev: false,
            zone_id: None,
        };
        assert!(kv::get_namespace_id(&target_with_dup_kv_bindings, "").is_err());
    }
}
