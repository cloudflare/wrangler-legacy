use crate::settings::global_user::GlobalUser;
use crate::terminal::{emoji, message};

pub fn whoami(user: &GlobalUser) -> Result<(), failure::Error> {
    // If using email + API key for auth, simply prints out email from config file.
    let email: String = match user {
        GlobalUser::KeyAuthUser { email, api_key: _ } => email.to_string(),
        GlobalUser::TokenAuthUser { api_token: _ } => failure::bail!(
            "At the moment, Wrangler cannot get user information for users using API tokens"
        ),
    };

    let msg = format!(
        "{} You are logged in with the email '{}'.",
        emoji::WAVING,
        email
    );
    message::info(&msg);
    Ok(())
}
