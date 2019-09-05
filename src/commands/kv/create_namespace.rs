use cloudflare::endpoints::workerskv::create_namespace::CreateNamespace;
use cloudflare::endpoints::workerskv::create_namespace::CreateNamespaceParams;
use cloudflare::framework::apiclient::ApiClient;

use crate::commands::kv;
use crate::settings::global_user::GlobalUser;
use crate::settings::project::Project;
use crate::terminal::message;

pub fn create_namespace(
    project: &Project,
    user: GlobalUser,
    title: &str,
) -> Result<(), failure::Error> {
    let client = kv::api_client(user)?;

    let msg = format!("Creating namespace with title \"{}\"", title);
    message::working(&msg);

    let response = client.request(&CreateNamespace {
        account_identifier: &project.account_id,
        params: CreateNamespaceParams {
            title: title.to_string(),
        },
    });

    match response {
        Ok(success) => message::success(&format!("Success: {:#?}", success.result)),
        Err(e) => kv::print_error(e),
    }

    Ok(())
}
