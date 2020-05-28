use cloudflare::endpoints::workerskv::WorkersKvNamespace;

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
