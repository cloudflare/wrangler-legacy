use cloudflare::endpoints::workerskv::create_namespace::CreateNamespace;
use cloudflare::endpoints::workerskv::create_namespace::CreateNamespaceParams;
use cloudflare::endpoints::workerskv::list_namespaces::ListNamespaces;
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
    let client = kv::api_client(user.to_owned())?;

    let title = match preview {
        false => format!("__{}-{}", target.name, "workers_sites_assets"),
        true => format!("__{}-{}", target.name, "workers_sites_assets_preview"),
    };
    let msg = format!("Creating namespace for Workers Site \"{}\"", title);
    message::working(&msg);

    let response = client.request(&CreateNamespace {
        account_identifier: &target.account_id,
        params: CreateNamespaceParams {
            title: title.to_owned(),
        },
    });

    match response {
        Ok(success) => Ok(success.result),
        Err(e) => match e {
            ApiFailure::Error(_status, api_errors) => {
                if api_errors.errors.iter().any(|e| e.code == 10014) {
                    log::info!("Namespace {} already exists.", title);
                    let response = client.request(&ListNamespaces {
                        account_identifier: &target.account_id,
                    });

                    match response {
                        Ok(success) => Ok(success
                            .result
                            .iter()
                            .find(|ns| ns.title == title)
                            .unwrap()
                            .to_owned()),
                        Err(e) => failure::bail!("{:?}", e),
                    }
                } else {
                    failure::bail!("{:?}", api_errors.errors)
                }
            }
            ApiFailure::Invalid(reqwest_err) => failure::bail!("Error: {}", reqwest_err),
        },
    }
}
