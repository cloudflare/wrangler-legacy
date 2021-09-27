use cloudflare::endpoints::workers::{ListBindings, WorkersBinding};
use cloudflare::framework::apiclient::ApiClient;
use cloudflare::framework::response::ApiErrors;
use cloudflare::framework::response::ApiFailure;
use cloudflare::framework::HttpApiClient;

use crate::http;
use crate::terminal::message::{Message, StdOut};

// Fetches all bindings for a script_name associated with an account_id
pub(crate) fn fetch_bindings(
    client: &HttpApiClient,
    account_id: &str,
    script_name: &str,
) -> Result<Vec<WorkersBinding>, anyhow::Error> {
    let response = client.request(&ListBindings {
        account_id,
        script_name,
    });
    match response {
        Ok(res) => Ok(res.result),
        Err(e) => {
            match e {
                ApiFailure::Error(_, ref api_errors) => {
                    api_error_info(api_errors);
                }
                ApiFailure::Invalid(_) => StdOut::info("Something went wrong in processing a request. Please consider raising an issue at https://github.com/cloudflare/wrangler/issues"),
            }
            anyhow::bail!(http::format_error(e, None))
        }
    }
}

fn api_error_info(api_errors: &ApiErrors) {
    let error = &api_errors.errors[0];
    if error.code == 9109 {
        // 9109 error code = Invalid access token
        StdOut::info("Your API/OAuth token might be expired, or might not have the necessary permissions. Please re-authenticate wrangler by running `wrangler login` or `wrangler config`.");
    } else if error.code == 6003 {
        // 6003 error code = Invalid request headers. A common case is when the value of an authorization method has been changed outside of wrangler commands
        StdOut::info("Your authentication method might be corrupted (e.g. API token value has been altered). Please re-authenticate wrangler by running `wrangler login` or `wrangler config`.");
    }
}
