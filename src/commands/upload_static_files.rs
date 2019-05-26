use crate::settings::global_user::GlobalUser;

pub fn upload_static_files(user: &GlobalUser, namespace: &str, directory: &str) {
    println!("Uploading {} to {}", directory, namespace);
}
