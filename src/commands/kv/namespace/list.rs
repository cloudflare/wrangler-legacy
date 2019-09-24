use cloudflare::endpoints::workerskv::list_namespaces::ListNamespaces;
use cloudflare::framework::apiclient::ApiClient;

use crate::commands::kv;
use crate::settings::global_user::GlobalUser;
use crate::settings::target::Target;

pub fn list(target: &Target, user: GlobalUser) -> Result<(), failure::Error> {
    kv::validate_target(target)?;
    let client = kv::api_client(user)?;

    let response = client.request(&ListNamespaces {
        account_identifier: &target.account_id,
    });

    match response {
        Ok(success) => {
            let result = serde_json::to_string(&success.result)?;
            println!("{}", result);
        }
        Err(e) => print!("{}", kv::format_error(e)),
    }

    Ok(())
}
