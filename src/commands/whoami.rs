use crate::http;
use crate::settings::global_user::GlobalUser;
use crate::terminal::message::{Message, StdOut};
use crate::terminal::{emoji, styles};
use cloudflare::endpoints::account::{self, Account};
use cloudflare::endpoints::user::GetUserDetails;
use cloudflare::framework::apiclient::ApiClient;
use cloudflare::framework::response::ApiFailure;

use anyhow::Result;
use prettytable::{Cell, Row, Table};

/// Return a string representing the token type based on user
fn get_token_type(
    user: &GlobalUser,
    missing_permissions: &mut Vec<String>,
    token_type: &str,
) -> Result<String> {
    if let Some(token_auth_email) = fetch_auth_token_email(user, missing_permissions)? {
        Ok(format!(
            "an {} Token, associated with the email '{}'",
            token_type, token_auth_email,
        ))
    } else {
        let wrangler_login_msg = styles::highlight("`wrangler login`");
        let wrangler_config_msg = styles::highlight("`wrangler config`");
        anyhow::bail!("Failed to retrieve information about the email associated with {} token. Please run {} or {}.", token_type, wrangler_login_msg, wrangler_config_msg)
    }
}

/// Tells the user who they are
pub fn whoami(user: &GlobalUser) -> Result<()> {
    let mut missing_permissions: Vec<String> = Vec::with_capacity(2);
    // Attempt to print email for both GlobalKeyAuth and TokenAuth users
    let auth: String = match user {
        GlobalUser::GlobalKeyAuth { email, .. } => {
            format!("a Global API Key, associated with the email '{}'", email,)
        }
        GlobalUser::ApiTokenAuth { .. } => get_token_type(user, &mut missing_permissions, "API")?,
        GlobalUser::OAuthTokenAuth { .. } => {
            get_token_type(user, &mut missing_permissions, "OAuth")?
        }
    };

    let accounts = fetch_accounts(user)?;
    let table = format_accounts(user, accounts, &mut missing_permissions);
    let mut msg = format!("{} You are logged in with {}!\n", emoji::WAVING, auth);
    let num_permissions_missing = missing_permissions.len();
    if num_permissions_missing > 0 {
        let login_msg = styles::highlight("`wrangler login`");
        let config_msg = styles::highlight("`wrangler config`");
        let whoami_msg = styles::highlight("`wrangler whoami`");
        if missing_permissions.len() == 1 {
            msg.push_str(&format!(
                "\nYour token is missing the '{}' permission.",
                styles::highlight(missing_permissions.get(0).unwrap())
            ));
        } else if missing_permissions.len() == 2 {
            msg.push_str(&format!(
                "\nYour token is missing the '{}' and '{}' permissions.",
                styles::highlight(missing_permissions.get(0).unwrap()),
                styles::highlight(missing_permissions.get(1).unwrap())
            ));
        }
        msg.push_str(&format!("\n\nPlease generate a new token and authenticate with {} or {}\nfor more information when running {}", login_msg, config_msg, whoami_msg));
    }

    StdOut::billboard(&msg);

    if table.len() > 1 {
        println!("{}", &table);
    }
    Ok(())
}

/// Print information either containing the user's account IDs,
/// or at least tell them where to get them.
pub fn display_account_id_maybe() {
    let account_id_msg = styles::highlight("account_id");
    let mut showed_account_id = false;

    if let Ok(user) = GlobalUser::new() {
        if let Ok(accounts) = fetch_accounts(&user) {
            let mut missing_permissions = Vec::with_capacity(2);
            let table = format_accounts(&user, accounts, &mut missing_permissions);
            if missing_permissions.is_empty() {
                StdOut::help(&format!("You can copy your {} below", account_id_msg));
                // table includes a newline so just `print!()` is fine
                print!("{}", &table);
                showed_account_id = true;
            }
        }
    }
    if !showed_account_id {
        StdOut::help(&format!(
            "You can find your {} in the right sidebar of your account's Workers page",
            account_id_msg
        ));
    }
}

fn fetch_auth_token_email(
    user: &GlobalUser,
    missing_permissions: &mut Vec<String>,
) -> Result<Option<String>> {
    let client = http::cf_v4_client(user)?;
    let response = client.request(&GetUserDetails {});
    match response {
        Ok(res) => Ok(Some(res.result.email)),
        Err(e) => match e {
            ApiFailure::Error(_, api_errors) => {
                let error = &api_errors.errors[0];

                if error.code == 9109 {
                    missing_permissions.push("User Details: Read".to_string());
                }
                Ok(None)
            }
            ApiFailure::Invalid(_) => anyhow::bail!(http::format_error(e, None)),
        },
    }
}

/// Fetch the accounts associated with a user
pub(crate) fn fetch_accounts(user: &GlobalUser) -> Result<Vec<Account>> {
    let client = http::cf_v4_client(user)?;
    let response = client.request(&account::ListAccounts { params: None });
    match response {
        Ok(res) => Ok(res.result),
        Err(e) => {
            match e {
                ApiFailure::Error(_, ref api_errors) => {
                    let error = &api_errors.errors[0];
                    if error.code == 9109 {
                        // 9109 error code = Invalid access token
                        StdOut::info("Your API token might be expired, or might not have the necessary permissions. Please re-authenticate wrangler by running `wrangler login` or `wrangler config`.");
                    } else if error.code == 6003 {
                        // 6003 error code = Invalid request headers. A common case is when the value of an authorization method has been changed outside of wrangler commands
                        StdOut::info("Your authentication method might be corrupted (e.g. API token value has been altered). Please re-authenticate wrangler by running `wrangler login` or `wrangler config`.");
                    }

                }
                ApiFailure::Invalid(_) => StdOut::info("Something went wrong in processing a request. Please consider raising an issue at https://github.com/cloudflare/wrangler/issues"),
            }

            anyhow::bail!(http::format_error(e, None))
        }
    }
}

/// Format a user's accounts into a nice table
fn format_accounts(
    user: &GlobalUser,
    accounts: Vec<Account>,
    missing_permissions: &mut Vec<String>,
) -> Table {
    let mut table = Table::new();
    let table_head = Row::new(vec![Cell::new("Account Name"), Cell::new("Account ID")]);
    table.add_row(table_head);

    match user {
        GlobalUser::GlobalKeyAuth { .. } => (),
        _ => {
            if accounts.is_empty() {
                missing_permissions.push("Account Settings: Read".to_string());
            }
        }
    }

    for account in accounts {
        let row = Row::new(vec![Cell::new(&account.name), Cell::new(&account.id)]);
        table.add_row(row);
    }
    table
}
