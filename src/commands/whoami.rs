use crate::user::User;

pub fn whoami(user: &User) {
    let user = &user.data;

    println!("👋 You are logged with the email '{}'.", user.email);
}
