use std::ffi::OsString;
use std::path::Path;

use cloudflare::framework::auth::Credentials;
use cloudflare::framework::response::ApiFailure;
use cloudflare::framework::HttpApiClient;
use failure::bail;
use http::status::StatusCode;

use crate::settings;
use crate::terminal::message;

mod create_namespace;
mod delete_bulk;
mod delete_key;
mod delete_namespace;
mod list_keys;
mod list_namespaces;
mod read_key;
mod rename_namespace;
mod write_bulk;
mod write_key;

pub use create_namespace::create_namespace;
pub use delete_bulk::delete_bulk;
pub use delete_key::delete_key;
pub use delete_namespace::delete_namespace;
pub use list_keys::list_keys;
pub use list_namespaces::list_namespaces;
pub use read_key::read_key;
pub use rename_namespace::rename_namespace;
pub use write_bulk::write_bulk;
pub use write_key::write_key;

fn api_client() -> Result<HttpApiClient, failure::Error> {
    let user = settings::global_user::GlobalUser::new()?;

    Ok(HttpApiClient::new(Credentials::from(user)))
}

fn account_id() -> Result<String, failure::Error> {
    let project = settings::project::Project::new()?;
    // we need to be certain that account id is present to make kv calls
    if project.account_id.is_empty() {
        bail!("Your wrangler.toml is missing the account_id field which is required to create KV namespaces!");
    }
    Ok(project.account_id)
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

// Courtesy of Steve Kalabnik's PoC :) Used for bulk operations (write, delete)
fn generate_key(path: &Path, directory: &Path) -> Result<String, failure::Error> {
    let path = path.strip_prefix(directory).unwrap();

    // next, we have to re-build the paths: if we're on Windows, we have paths with
    // `\` as separators. But we want to use `/` as separators. Because that's how URLs
    // work.
    let mut path_with_forward_slash = OsString::new();

    for (i, component) in path.components().enumerate() {
        // we don't want a leading `/`, so skip that
        if i > 0 {
            path_with_forward_slash.push("/");
        }

        path_with_forward_slash.push(component);
    }

    // if we have a non-utf8 path here, it will fail, but that's not realistically going to happen
    let path = path_with_forward_slash.to_str().expect(&format!(
        "found a non-UTF-8 path, {:?}",
        path_with_forward_slash
    ));

    Ok(path.to_string())
}
