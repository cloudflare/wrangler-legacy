use cloudflare::endpoints::workerskv::create_namespace::{CreateNamespace, CreateNamespaceParams};
use cloudflare::endpoints::workerskv::WorkersKvNamespace;
use cloudflare::framework::apiclient::ApiClient;
use cloudflare::framework::response::ApiFailure;

use crate::commands::kv;
use crate::settings::global_user::GlobalUser;
use crate::settings::target::Target;
use crate::terminal::message;

pub fn site(
    target: &Target,
    user: &GlobalUser,
    preview: bool,
) -> Result<WorkersKvNamespace, failure::Error> {
    kv::validate_target(target)?;

    let title = if preview {
        format!("__{}-{}", target.name, "workers_sites_assets_preview")
    } else {
        format!("__{}-{}", target.name, "workers_sites_assets")
    };

    // We call CreateNamespace here directly because we want raw access to the
    // response codes; the implementations in commands::kv::namespace cannot surface
    // the raw response
    let client = kv::api_client(user)?;
    let response = client.request(&CreateNamespace {
        account_identifier: &target.account_id,
        params: CreateNamespaceParams {
            title: title.to_owned(),
        },
    });

    match response {
        Ok(success) => {
            let msg = format!("Created namespace for Workers Site \"{}\"", title);
            message::working(&msg);
            Ok(success.result)
        }
        Err(e) => match e {
            ApiFailure::Error(_status, api_errors) => {
                if api_errors.errors.iter().any(|e| e.code == 10026) {
                    failure::bail!("You will need to enable Workers Unlimited for your account before you can use this feature.")
                } else if api_errors.errors.iter().any(|e| e.code == 10014) {
                    log::info!("Namespace {} already exists.", title);
                    let namespaces = kv::namespace::list::call_api(target, user)?;

                    let msg = format!("Using namespace for Workers Site \"{}\"", title);
                    message::working(&msg);

                    Ok(namespaces
                        .iter()
                        .find(|ns| ns.title == title)
                        .unwrap()
                        .to_owned())
                } else {
                    failure::bail!("{:?}", api_errors.errors)
                }
            }
            ApiFailure::Invalid(reqwest_err) => failure::bail!("Error: {}", reqwest_err),
        },
    }
}
