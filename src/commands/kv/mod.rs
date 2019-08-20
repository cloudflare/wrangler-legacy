use cloudflare::framework::auth::Credentials;
use cloudflare::framework::response::ApiFailure;

use cloudflare::framework::HttpApiClient;

use crate::settings;
use crate::terminal::message;

mod create_namespace;
mod delete_namespace;
mod list_namespaces;
mod rename_namespace;
mod write_key;

pub use create_namespace::create_namespace;
pub use delete_namespace::delete_namespace;
pub use list_namespaces::list_namespaces;
pub use rename_namespace::rename_namespace;
pub use write_key::write_key;

fn api_client() -> Result<HttpApiClient, failure::Error> {
    let user = settings::global_user::GlobalUser::new()?;

    Ok(HttpApiClient::new(Credentials::from(user)))
}

fn account_id() -> Result<String, failure::Error> {
    let project = settings::project::Project::new()?;
    // we need to be certain that account id is present to make kv calls
    if project.account_id.is_empty() {
        panic!("Your wrangler.toml is missing the account_id field which is required to create KV namespaces!");
    }
    Ok(project.account_id)
}

fn print_error(e: ApiFailure) {
    match e {
        ApiFailure::Error(_status, api_errors) => {
            for error in api_errors.errors {
                message::warn(&format!("Error {}: {}", error.code, error.message,));

                let suggestion = help(error.code);
                if !suggestion.is_empty() {
                    message::help(suggestion);
                }
            }
        }
        ApiFailure::Invalid(reqwest_err) => message::warn(&format!("Error: {}", reqwest_err)),
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
        10017 | 10026 => "Check your account settings in the Cloudflare dashboard",
        _ => "",
    }
}
