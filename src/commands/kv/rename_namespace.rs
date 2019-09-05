use cloudflare::endpoints::workerskv::rename_namespace::RenameNamespace;
use cloudflare::endpoints::workerskv::rename_namespace::RenameNamespaceParams;
use cloudflare::framework::apiclient::ApiClient;

use crate::commands::kv;
use crate::settings::global_user::GlobalUser;
use crate::settings::project::Project;
use crate::terminal::message;

pub fn rename_namespace(
    project: &Project,
    user: GlobalUser,
    id: &str,
    title: &str,
) -> Result<(), failure::Error> {
    let client = kv::api_client(user)?;

    let msg = format!("Renaming namespace {} to have title \"{}\"", id, title);
    message::working(&msg);

    let response = client.request(&RenameNamespace {
        account_identifier: &project.account_id,
        namespace_identifier: &id,
        params: RenameNamespaceParams {
            title: title.to_string(),
        },
    });

    match response {
        Ok(_success) => message::success("Success"),
        Err(e) => kv::print_error(e),
    }

    Ok(())
}
