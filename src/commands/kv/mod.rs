use cloudflare::auth::Credentials;
use cloudflare::response::APIFailure;
use cloudflare::response::APIResponse;
use cloudflare::response::APIResult;
use cloudflare::HTTPAPIClient;

use crate::settings;
use crate::terminal::message;

mod create_namespace;

pub use create_namespace::create_namespace;

fn api_client() -> Result<HTTPAPIClient, failure::Error> {
    let user = settings::global_user::GlobalUser::new()?;

    Ok(HTTPAPIClient::new(Credentials::from(user)))
}

fn account_id() -> Result<String, failure::Error> {
    let project = settings::project::Project::new()?;
    // we need to be certain that account id is present to make kv calls
    if project.account_id.is_empty() {
        panic!("Your wrangler.toml is missing the account_id field which is required to create KV namespaces!");
    }
    Ok(project.account_id)
}

fn print_response<T: APIResult>(response: APIResponse<T>) {
    match response {
        Ok(success) => message::success(&format!("Success: {:#?}", success.result)),
        Err(e) => match e {
            APIFailure::Error(_status, errors) => {
                for error in errors {
                    message::warn(&format!("Error {}: {}", error.code, error.message,));

                    let suggestion = help(error.code);
                    if !suggestion.is_empty() {
                        message::help(suggestion);
                    }
                }
            }
            APIFailure::Invalid(reqwest_err) => message::warn(&format!("Error: {}", reqwest_err)),
        },
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
