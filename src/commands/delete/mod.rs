use cloudflare::endpoints::workers::DeleteScript;
use cloudflare::framework::apiclient::ApiClient;
use cloudflare::framework::response::ApiFailure;

use crate::http;
use crate::settings::global_user::GlobalUser;
use crate::terminal::interactive;
use crate::terminal::message::{Message, StdOut};

fn format_error(e: ApiFailure) -> String {
    http::format_error(e, Some(&script_errors))
}

// secret_errors() provides more detailed explanations of API error codes.
fn script_errors(error_code: u16) -> &'static str {
    match error_code {
        10000 => "Your authentication might be expired or invalid. Please run `wrangler login` or `wrangler config` to authorize Wrangler",
        10007 => "The script could not be found. Please make sure that that the script being deleted exists.",
        _ => "",
    }
}

pub fn run(
    user: &GlobalUser,
) -> Result<(), anyhow::Error> {
    Ok(())
}


pub fn delete_script(
    user: &GlobalUser,
    force: bool,
    account_id: &str,
    script_id: &str,
) -> Result<(), anyhow::Error> {
    if !force {
        match interactive::confirm(&format!("Are you sure you want to permanently delete the script name \"{}\" from the account ID {}?", script_id, account_id)) {
            Ok(true) => (),
            Ok(false) => {
                StdOut::info(&format!("Not deleting script \"{}\"", script_id));
                return Ok(());
            },
            Err(e) => anyhow::bail!(e),
        }
    }

    StdOut::working(&format!(
        "Deleting the script \"{}\" on account {}.",
        script_id, account_id
    ));

    let client = http::cf_v4_client(user)?;

    let response = client.request(&DeleteScript {
        account_id,
        script_id,
    });

    println!("response: {:?}", response);
    match response {
        Ok(_) => {
            StdOut::success(&format!("Success! Deleted script \"{}\".", script_id));
        },
        Err(e) => {
            println!("ERROR");
            anyhow::bail!(format_error(e))
        },
    }

    Ok(())
}
