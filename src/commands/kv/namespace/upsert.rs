use cloudflare::endpoints::workerskv::WorkersKvNamespace;
use cloudflare::framework::apiclient::ApiClient;
use cloudflare::framework::response::ApiFailure;

use crate::commands::kv;
use crate::settings::global_user::GlobalUser;
use crate::settings::toml::Target;
use crate::terminal::message;

/// create a new namespace if it doesn't exist
/// otherwise, return the already existing namespace
pub fn upsert(
    target: &Target,
    user: &GlobalUser,
    namespace_title: &str,
) -> Result<WorkersKvNamespace, failure::Error> {
    kv::validate_target(target)?;

    let client = kv::api_client(user)?;
    let response = kv::namespace::create::call_api(&client, target, namespace_title);

    match response {
        Ok(success) => {
            let msg = format!("Created namespace \"{}\"", namespace_title);
            message::working(&msg);
            Ok(success.result)
        }
        Err(e) => match e {
            ApiFailure::Error(_status, api_errors) => {
                if api_errors.errors.iter().any(|e| e.code == 10026) {
                    failure::bail!("You will need to enable Workers Unlimited for your account before you can use this feature.")
                } else if api_errors.errors.iter().any(|e| e.code == 10014) {
                    log::info!("Namespace {} already exists.", namespace_title);

                    let msg = format!("Using namespace \"{}\"", namespace_title);
                    message::working(&msg);

                    get_namespace_from_title(&client, target, namespace_title)
                } else {
                    failure::bail!("{:?}", api_errors.errors)
                }
            }
            ApiFailure::Invalid(reqwest_err) => failure::bail!("Error: {}", reqwest_err),
        },
    }
}

fn get_namespace_from_title(
    client: &impl ApiClient,
    target: &Target,
    title: &str,
) -> Result<WorkersKvNamespace, failure::Error> {
    let result = kv::namespace::list::call_api(client, target);

    match result {
        Ok(success) => Ok(success
            .result
            .iter()
            .find(|ns| ns.title == title)
            .unwrap()
            .to_owned()),
        Err(e) => failure::bail!(kv::format_error(e)),
    }
}
