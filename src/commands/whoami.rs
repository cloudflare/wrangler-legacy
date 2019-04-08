use crate::user::User;

pub fn whoami(user: &User) {
    let user = &user.data;

    println!(
        "👋 You are logged in as {} {}, with the email '{}'.",
        user.first_name, user.last_name, user.email
    );
}
