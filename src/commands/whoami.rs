use crate::settings::global_user::GlobalUser;
use crate::terminal::{emoji, message};

pub fn whoami(user: &GlobalUser) {
    let msg = format!(
        "{} You are logged with the email '{}'.",
        emoji::WAVING,
        user.email
    );

    message::info(&msg);
}
