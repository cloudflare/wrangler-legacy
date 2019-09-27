use cloudflare::endpoints::workerskv::create_namespace::{CreateNamespace, CreateNamespaceParams};
use cloudflare::endpoints::workerskv::list_namespaces::{ListNamespaces, ListNamespacesParams};
use cloudflare::endpoints::workerskv::WorkersKvNamespace;
use cloudflare::framework::apiclient::ApiClient;
use cloudflare::framework::response::ApiFailure;

use crate::commands::kv;
use crate::settings::global_user::GlobalUser;
use crate::settings::target::Target;
use crate::terminal::message;

pub const MAX_NAMESPACES_PER_PAGE: u32 = 100;
pub const PAGE_NUMBER: u32 = 1;

pub fn site(
    target: &Target,
    user: &GlobalUser,
    preview: bool,
) -> Result<WorkersKvNamespace, failure::Error> {
    kv::validate_target(target)?;
    let client = kv::api_client(user)?;

    let title = if preview {
        format!("__{}-{}", target.name, "workers_sites_assets_preview")
    } else {
        format!("__{}-{}", target.name, "workers_sites_assets")
    };

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
                    let params = ListNamespacesParams {
                        page: Some(PAGE_NUMBER),
                        per_page: Some(MAX_NAMESPACES_PER_PAGE),
                    };

                    let response = client.request(&ListNamespaces {
                        account_identifier: &target.account_id,
                        params: params,
                    });

                    match response {
                        Ok(success) => {
                            let msg = format!("Using namespace for Workers Site \"{}\"", title);
                            message::working(&msg);

                            Ok(success
                                .result
                                .iter()
                                .find(|ns| ns.title == title)
                                .unwrap()
                                .to_owned())
                        }
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
