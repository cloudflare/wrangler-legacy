use cloudflare::endpoints::workerskv::WorkersKvNamespace;
use cloudflare::framework::apiclient::ApiClient;
use cloudflare::framework::response::ApiFailure;

use crate::commands::kv;
use crate::http;
use crate::settings::global_user::GlobalUser;
use crate::settings::toml::Target;
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

    let client = kv::api_client(user)?;
    let response = kv::namespace::create::call_api(&client, target, &title);

    match response {
        Ok(success) => {
            let msg = format!("Created namespace for Workers Site \"{}\"", title);
            message::working(&msg);
            Ok(success.result)
        }
        Err(e) => match &e {
            ApiFailure::Error(_status, api_errors) => {
                if api_errors.errors.iter().any(|e| e.code == 10014) {
                    log::info!("Namespace {} already exists.", title);

                    let msg = format!("Using namespace for Workers Site \"{}\"", title);
                    message::working(&msg);

                    get_id_from_namespace_list(&client, target, &title)
                } else {
                    failure::bail!("{}", http::format_error(e, Some(&error_suggestions)))
                }
            }
            _ => failure::bail!("{}", http::format_error(e, Some(&error_suggestions))),
        },
    }
}

fn get_id_from_namespace_list(
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
        Err(e) => failure::bail!("{}", http::format_error(e, Some(&error_suggestions))),
    }
}

fn error_suggestions(code: u16) -> &'static str {
    match code {
        10026 => "You will need to enable Workers Unlimited for your account before you can use this feature.",
        10014 => "Namespace already exists, try using a different namespace.",
        10037 => "Edit your API Token to have correct permissions, or use the 'Edit Cloudflare Workers' API Token template.",
        _ => "",
    }
}
