use cloudflare::apiclient::APIClient;
use cloudflare::response::APIFailure;
use cloudflare::workerskv::create_namespace as kv_api;

use crate::settings::global_user::GlobalUser;
use crate::settings::project::Project;

use super::{api_errors_to_message, new_api_client, KvNamespace};

pub fn create_namespace(
    user: GlobalUser,
    project: &Project,
    title: String,
) -> Result<KvNamespace, failure::Error> {
    let client = new_api_client(user);

    let res = client
        .request(&kv_api::CreateNamespace {
            account_identifier: &project.account_id,
            params: kv_api::CreateNamespaceParams { title },
        })
        .unwrap_or_else(|failure| match failure {
            APIFailure::Error(_, errors) => {
                panic!("{}", api_errors_to_message(errors));;
            }
            APIFailure::Invalid(error) => {
                panic!("{}", error);
            }
        });

    Ok(KvNamespace::from(res))
}
