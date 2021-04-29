use cloudflare::endpoints::workerskv::WorkersKvNamespace;
use cloudflare::framework::response::ApiFailure;

use anyhow::Result;

use crate::http;
use crate::settings::global_user::GlobalUser;
use crate::settings::toml::Target;

use super::create;
use super::list;

pub enum UpsertedNamespace {
    Created(WorkersKvNamespace),
    Reused(WorkersKvNamespace),
}

pub fn upsert(target: &Target, user: &GlobalUser, title: String) -> Result<UpsertedNamespace> {
    let client = http::cf_v4_client(user)?;
    let response = create(&client, &target.account_id, &title);

    match response {
        Ok(success) => Ok(UpsertedNamespace::Created(success.result)),
        Err(e) => match &e {
            ApiFailure::Error(_status, api_errors) => {
                if api_errors.errors.iter().any(|e| e.code == 10014) {
                    log::info!("Namespace {} already exists.", title);

                    match list(&client, target)?
                        .iter()
                        .find(|ns| ns.title == title) {
                        Some(namespace) => Ok(UpsertedNamespace::Reused(namespace.to_owned())),
                        None => anyhow::bail!("namespace already exists, but could not be found in the API's listed namespaces"),
                    }
                } else {
                    anyhow::bail!("{}", http::format_error(e, Some(&error_suggestions)))
                }
            }
            _ => anyhow::bail!("{}", http::format_error(e, Some(&error_suggestions))),
        },
    }
}

fn error_suggestions(code: u16) -> &'static str {
    match code {
        10014 => "Namespace already exists, try using a different namespace.",
        10037 => "Edit your API Token to have correct permissions, or use the 'Edit Cloudflare Workers' API Token template.",
        _ => "",
    }
}
