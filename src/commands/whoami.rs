use crate::user::User;
use crate::emoji;

pub fn whoami(user: &User) {
    let user = &user.data;

    println!("{} You are logged with the email '{}'.", emoji::WAVING, user.email);
}
