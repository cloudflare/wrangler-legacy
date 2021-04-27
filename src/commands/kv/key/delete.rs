use cloudflare::endpoints::workerskv::delete_key::DeleteKey;
use cloudflare::framework::apiclient::ApiClient;

use anyhow::Result;

use crate::commands::kv::{format_error, validate_target};
use crate::http;
use crate::settings::global_user::GlobalUser;
use crate::settings::toml::Target;
use crate::terminal::interactive;
use crate::terminal::message::{Message, StdOut};
pub fn delete(target: &Target, user: &GlobalUser, id: &str, key: &str) -> Result<()> {
    validate_target(target)?;
    let client = http::cf_v4_client(user)?;

    match interactive::confirm(&format!("Are you sure you want to delete key \"{}\"?", key)) {
        Ok(true) => (),
        Ok(false) => {
            StdOut::info(&format!("Not deleting key \"{}\"", key));
            return Ok(());
        }
        Err(e) => anyhow::bail!(e),
    }

    let msg = format!("Deleting key \"{}\"", key);
    StdOut::working(&msg);

    let response = client.request(&DeleteKey {
        account_identifier: &target.account_id,
        namespace_identifier: id,
        key, // this is url encoded within cloudflare-rs
    });

    match response {
        Ok(_) => StdOut::success("Success"),
        Err(e) => print!("{}", format_error(e)),
    }

    Ok(())
}
