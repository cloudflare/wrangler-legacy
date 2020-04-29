use std::collections::HashSet;
use std::time::Duration;

use cloudflare::framework::auth::Credentials;
use cloudflare::framework::response::ApiFailure;
use cloudflare::framework::{Environment, HttpApiClient, HttpApiClientConfig};

use percent_encoding::{percent_encode, PATH_SEGMENT_ENCODE_SET};

use crate::http::{self, feature::headers};
use crate::settings::global_user::GlobalUser;
use crate::settings::toml::{KvNamespace, Target};

pub mod bucket;
pub mod bulk;
pub mod key;
pub mod namespace;

// Create a special API client that has a longer timeout than usual, given that KV operations
// can be lengthy if payloads are large.
fn api_client(user: &GlobalUser) -> Result<HttpApiClient, failure::Error> {
    let config = HttpApiClientConfig {
        http_timeout: Duration::from_secs(5 * 60),
        default_headers: headers(None),
    };

    HttpApiClient::new(
        Credentials::from(user.to_owned()),
        config,
        Environment::Production,
    )
}

fn format_error(e: ApiFailure) -> String {
    http::format_error(e, Some(&kv_help))
}

// kv_help() provides more detailed explanations of Workers KV API error codes.
// See https://api.cloudflare.com/#workers-kv-namespace-errors for details.
fn kv_help(error_code: u16) -> &'static str {
    match error_code {
        7003 | 7000 => {
            "Your wrangler.toml is likely missing the field \"account_id\", which is required to write to Workers KV."
        }
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

fn check_duplicate_namespaces(kv_namespaces: &[KvNamespace]) -> bool {
    // HashSet for detecting duplicate namespace bindings
    let mut binding_names: HashSet<String> = HashSet::new();

    for namespace in kv_namespaces {
        // Check if this is a duplicate binding
        if binding_names.contains(&namespace.binding) {
            return true;
        } else {
            binding_names.insert(namespace.binding.clone());
        }
    }
    false
}

// Get namespace id for a given binding name.
pub fn get_namespace_id_from_binding(
    kv_namespaces: &[KvNamespace],
    binding: &str,
) -> Result<String, failure::Error> {
    if check_duplicate_namespaces(kv_namespaces) {
        failure::bail!("Namespace binding \"{}\" is duplicated", binding,)
    }

    for namespace in kv_namespaces {
        if namespace.binding == binding {
            return Ok(namespace.id.to_string());
        }
    }

    failure::bail!("Namespace binding \"{}\" not found", binding,)
}

fn url_encode_key(key: &str) -> String {
    percent_encode(key.as_bytes(), PATH_SEGMENT_ENCODE_SET).to_string()
}

#[cfg(test)]
mod tests {
    use crate::commands::kv;
    use crate::settings::toml::{KvNamespace, Target, TargetType};

    #[test]
    fn it_can_detect_duplicate_bindings() {
        let duplicate_namespace = KvNamespace {
            id: "fake".to_string(),
            binding: "KV".to_string(),
            bucket: None,
        };
        let kv_namespaces = vec![duplicate_namespace.clone(), duplicate_namespace];
        assert!(kv::get_namespace_id_from_binding(&kv_namespaces, "").is_err());
    }
}
