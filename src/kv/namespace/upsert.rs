use cloudflare::endpoints::workerskv::WorkersKvNamespace;
use cloudflare::framework::response::ApiFailure;

use crate::http;
use crate::settings::global_user::GlobalUser;
use crate::settings::toml::Target;

use super::create;
use super::list;

pub enum UpsertedNamespace {
    Created(WorkersKvNamespace),
    Reused(WorkersKvNamespace),
}

pub fn upsert(
    target: &Target,
    user: &GlobalUser,
    title: String,
) -> Result<UpsertedNamespace, failure::Error> {
    let client = http::cf_v4_client(user)?;
    let response = create(&client, target, &title);

    match response {
        Ok(success) => Ok(UpsertedNamespace::Created(success.result)),
        Err(e) => match &e {
            ApiFailure::Error(_status, api_errors) => {
                if api_errors.errors.iter().any(|e| e.code == 10026) {
                    failure::bail!("{}", http::format_error(e, Some(&error_suggestions)))
                } else if api_errors.errors.iter().any(|e| e.code == 10014) {
                    log::info!("Namespace {} already exists.", title);

                    let namespace = list(&client, target)?
                        .result
                        .iter()
                        .find(|ns| ns.title == title)
                        .unwrap()
                        .to_owned();
                    Ok(UpsertedNamespace::Reused(namespace))
                } else {
                    failure::bail!("{}", http::format_error(e, Some(&error_suggestions)))
                }
            }
            _ => failure::bail!("{}", http::format_error(e, Some(&error_suggestions))),
        },
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
