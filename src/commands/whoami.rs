use crate::emoji;
use crate::settings::global_user::GlobalUser;

pub fn whoami(user: &GlobalUser) {
    println!(
        "{} You are logged with the email '{}'.",
        emoji::WAVING,
        user.email
    );
}
