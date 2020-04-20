use crate::http;
use crate::settings::global_user::GlobalUser;
use crate::terminal::{emoji, message};

use cloudflare::endpoints::account::{self, Account};
use cloudflare::endpoints::user::GetUserDetails;
use cloudflare::framework::apiclient::ApiClient;
use cloudflare::framework::response::ApiFailure;

use prettytable::{Cell, Row, Table};

pub fn whoami(user: &GlobalUser) -> Result<(), failure::Error> {
    // Attempt to print email for both GlobalKeyAuth and TokenAuth users
    let auth: String = match user {
        GlobalUser::GlobalKeyAuth { email, .. } => {
            format!("a Global API Key, associated with the email '{}'", email,)
        }
        GlobalUser::TokenAuth { .. } => {
            let token_auth_email = fetch_api_token_email(user)?;

            if let Some(token_auth_email) = token_auth_email {
                format!(
                    "an API Token, associated with the email '{}'",
                    token_auth_email,
                )
            } else {
                "an API Token".to_string()
            }
        }
    };

    println!("\n{} You are logged in with {}.\n", emoji::WAVING, auth,);
    let accounts = fetch_accounts(user)?;
    let table = format_accounts(user, accounts);
    println!("{}", &table);
    Ok(())
}

fn fetch_api_token_email(user: &GlobalUser) -> Result<Option<String>, failure::Error> {
    let client = http::cf_v4_client(user)?;
    let response = client.request(&GetUserDetails {});
    match response {
        Ok(res) => Ok(Some(res.result.email)),
        Err(e) => match e {
            ApiFailure::Error(_, api_errors) => {
                let error = &api_errors.errors[0];
                if error.code == 9109 {
                    message::billboard("Your token is missing the 'User Details: Read' permission.\n\nPlease generate and auth with a new token that has these perms to be able to identify this token.\n");
                }
                Ok(None)
            }
            ApiFailure::Invalid(_) => failure::bail!(http::format_error(e, None)),
        },
    }
}

fn fetch_accounts(user: &GlobalUser) -> Result<Vec<Account>, failure::Error> {
    let client = http::cf_v4_client(user)?;
    let response = client.request(&account::ListAccounts { params: None });
    match response {
        Ok(res) => Ok(res.result),
        Err(e) => failure::bail!(http::format_error(e, None)),
    }
}

fn format_accounts(user: &GlobalUser, accounts: Vec<Account>) -> Table {
    let mut table = Table::new();
    let table_head = Row::new(vec![Cell::new("Account Name"), Cell::new("Account ID")]);
    table.add_row(table_head);

    if let GlobalUser::TokenAuth { .. } = user {
        if accounts.is_empty() {
            message::billboard("Your token is missing the 'Account Settings: Read' permission.\n\nPlease generate and auth with a new token that has these perms to be able to list your accounts.");
        }
    }

    for account in accounts {
        let row = Row::new(vec![Cell::new(&account.name), Cell::new(&account.id)]);
        table.add_row(row);
    }
    table
}
