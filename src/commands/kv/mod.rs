use std::collections::HashSet;
use std::time::Duration;

use cloudflare::framework::response::ApiFailure;
use cloudflare::framework::{HttpApiClient, HttpApiClientConfig};

use percent_encoding::{percent_encode, PATH_SEGMENT_ENCODE_SET};

use crate::settings::global_user::GlobalUser;
use crate::settings::target::Target;

use crate::http;
use crate::http::ErrorCodeDetail;

pub mod bucket;
pub mod bulk;
pub mod key;
pub mod namespace;

// Truncate all "yes", "no" responses for interactive delete prompt to just "y" or "n".
const INTERACTIVE_RESPONSE_LEN: usize = 1;
const YES: &str = "y";
const NO: &str = "n";

// Create a special API client that has a longer timeout than usual, given that KV operations
// can be lengthy if payloads are large.
fn api_client(user: &GlobalUser) -> Result<HttpApiClient, failure::Error> {
    http::api_client(
        user,
        HttpApiClientConfig {
            // Use 5 minute timeout instead of default 30-second one.
            // This is useful for bulk upload operations.
            http_timeout: Duration::from_secs(5 * 60),
        },
    )
}

fn format_error(e: ApiFailure) -> String {
    http::format_error(e, ErrorCodeDetail::WorkersKV)
}

pub fn validate_target(target: &Target) -> Result<(), failure::Error> {
    let mut missing_fields = Vec::new();

    if target.account_id.is_empty() {
        missing_fields.push("account_id")
    };

    if !missing_fields.is_empty() {
        failure::bail!(
            "Your wrangler.toml is missing the following field(s): {:?}",
            missing_fields
        )
    } else {
        Ok(())
    }
}

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
            zone_id: None,
            site: None,
        };
        assert!(kv::get_namespace_id(&target_with_dup_kv_bindings, "").is_err());
    }
}
