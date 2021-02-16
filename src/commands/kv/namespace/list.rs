extern crate serde_json;

use crate::commands::kv;
use crate::http;
use crate::kv::namespace::list;
use crate::settings::global_user::GlobalUser;
use crate::settings::toml::Target;

pub fn run(target: &Target, user: &GlobalUser) -> Result<(), failure::Error> {
    kv::validate_target(target)?;

    let client = http::cf_v4_client(user)?;
    let result = list(&client, target);
    match result {
        Ok(namespaces) => {
            println!("{}", serde_json::to_string(&namespaces)?);
        }
        Err(e) => failure::bail!(e),
    }
    Ok(())
}
