use crate::settings::global_user::GlobalUser;
use crate::terminal::{emoji, message};

pub fn whoami(user: &GlobalUser) {
    // Only returns email if present (using email + API key for auth).
    match &user.email {
        Some(email) => {
            let msg = format!(
                "{} You are logged with the email '{}'.",
                emoji::WAVING,
                email
            );

            message::info(&msg);
        }
        None => (),
    }
}
