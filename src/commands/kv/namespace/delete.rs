use crate::commands::kv;
use crate::http;
use crate::kv::namespace::delete;
use crate::settings::global_user::GlobalUser;
use crate::settings::toml::Target;
use crate::terminal::interactive;
use crate::terminal::message::{Message, StdOut};

use anyhow::Result;

pub fn run(target: &Target, user: &GlobalUser, id: &str) -> Result<()> {
    let client = http::cf_v4_client(user)?;

    match interactive::confirm(&format!(
        "Are you sure you want to delete namespace {}?",
        id
    )) {
        Ok(true) => (),
        Ok(false) => {
            StdOut::info(&format!("Not deleting namespace {}", id));
            return Ok(());
        }
        Err(e) => anyhow::bail!(e),
    }

    let msg = format!("Deleting namespace {}", id);
    StdOut::working(&msg);

    let response = delete(client, target.account_id.load()?, id);
    match response {
        Ok(_) => {
            StdOut::success("Success");
            StdOut::warn(
                "Make sure to remove this \"kv-namespace\" entry from your configuration file!",
            )
        }
        Err(e) => print!("{}", kv::format_error(e)),
    }

    Ok(())
}
