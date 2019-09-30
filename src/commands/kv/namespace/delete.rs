use cloudflare::endpoints::workerskv::remove_namespace::RemoveNamespace;
use cloudflare::framework::apiclient::ApiClient;

use crate::commands::kv;
use crate::settings::global_user::GlobalUser;
use crate::settings::target::Target;
use crate::terminal::message;

pub fn delete(target: &Target, user: &GlobalUser, id: &str) -> Result<(), failure::Error> {
    kv::validate_target(target)?;
    let client = kv::api_client(user);

    match kv::interactive_delete(&format!(
        "Are you sure you want to delete namespace {}?",
        id
    )) {
        Ok(true) => (),
        Ok(false) => {
            message::info(&format!("Not deleting namespace {}", id));
            return Ok(());
        }
        Err(e) => failure::bail!(e),
    }

    let msg = format!("Deleting namespace {}", id);
    message::working(&msg);

    let response = client.request(&RemoveNamespace {
        account_identifier: &target.account_id,
        namespace_identifier: id,
    });

    match response {
        Ok(_) => {
            message::success("Success");
            message::warn(
                "Make sure to remove this \"kv-namespace\" entry from your wrangler.toml!",
            )
        }
        Err(e) => print!("{}", kv::format_error(e)),
    }

    Ok(())
}
