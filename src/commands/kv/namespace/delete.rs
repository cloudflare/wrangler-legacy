use cloudflare::endpoints::workerskv::remove_namespace::RemoveNamespace;
use cloudflare::framework::apiclient::ApiClient;

use crate::commands::kv;
use crate::settings::global_user::GlobalUser;
use crate::settings::target::Target;
use crate::terminal::message;

pub fn delete(project: &Target, user: GlobalUser, id: &str) -> Result<(), failure::Error> {
    let client = kv::api_client(user)?;

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
        account_identifier: &project.account_id,
        namespace_identifier: id,
    });

    match response {
        Ok(_success) => message::success("Success"),
        Err(e) => kv::print_error(e),
    }

    Ok(())
}
