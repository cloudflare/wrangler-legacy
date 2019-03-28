use crate::user::User;

pub fn whoami(user: &User) {
    let user = &user.account;

    println!("👋 You are logged in as {}.", user.name);
}
