use cloudflare::endpoints::workerskv::list_namespaces::ListNamespaces;
use cloudflare::framework::apiclient::ApiClient;

use crate::commands::kv;
use crate::settings::global_user::GlobalUser;
use crate::settings::target::Target;

pub fn list(target: &Target, user: GlobalUser) -> Result<(), failure::Error> {
    let client = kv::api_client(user)?;

    let response = client.request(&ListNamespaces {
        account_identifier: &target.account_id,
    });

    match response {
        Ok(success) => {
            let result = serde_json::to_string(&success.result)?;
            println!("{}", result);
        }
        Err(e) => kv::print_error(e),
    }

    Ok(())
}
