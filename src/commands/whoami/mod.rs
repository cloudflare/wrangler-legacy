use crate::settings::global_user::GlobalUser;
use crate::terminal::{emoji, message};

pub fn whoami(user: &GlobalUser) -> Result<(), failure::Error> {
    // If using email + API key for auth, simply prints out email from config file.
    let email: String = match &user.email {
        Some(email) => email.to_string(),
        None => failure::bail!(
            "At the moment, Wrangler cannot get user information for users using API tokens"
        ),
    };

    let msg = format!(
        "{} You are logged with the email '{}'.",
        emoji::WAVING,
        email
    );
    message::info(&msg);
    Ok(())
}
