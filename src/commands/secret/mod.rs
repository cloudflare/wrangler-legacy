// use crate::http;
use crate::settings::global_user::GlobalUser;
use crate::settings::toml::Target;
use crate::terminal::{emoji, message};
use serde::{ Serialize};
// For interactively handling  reading in a string
pub fn interactive_get_string(prompt_string: &str) -> Result<bool, failure::Error> {
    println!("{}",prompt_string);
    let mut response: String = read!("{}\n");
    println!("{}",response);
    match response.as_str() {
        "" => Ok(false),
        _ =>  Ok(true),
    }
}

#[derive(Serialize)]
pub struct Secret {
    secret: String,
}
#[derive(Serialize)]
pub struct Script {
    secret: String,
}

impl Secret {

    pub fn put(name: &str, user: &GlobalUser) -> Result<(), failure::Error> {
        message::success(&format!("Success! You've uploaded secret {}.", name));
        Ok(())
    }
}
impl Script {
    pub fn put(name: &str, target: &Target, user: &GlobalUser) -> Result<(), failure::Error> {
        message::success(&format!("Success! You've bound the secret {} to {}.", name, target.name));
        Ok(())
    }
}



fn bind_secret(
    name: &str,
    user: &GlobalUser,
    target: &Target,
) -> Result<(), failure::Error> {
    let msg = format!(
        "Binding secret to script {}",
        target.name
    );
    message::working(&msg);
    Script::put(name, &target,  user)
}
fn put_secret(
    
    name: &str,
    user: &GlobalUser,
    target: &Target,
) -> Result<(), failure::Error> {
    let msg = format!(
        "Creating the secret for account {}",
        target.account_id
    );
    
    message::working(&msg);
    Secret::put(name,  user)
}

pub fn set_secret(name: &str, user: &GlobalUser, target: &Target) -> Result<(), failure::Error> {
  match interactive_get_string(&format!(
    "Enter the secret text you'd like assigned to {}?",
    name
    )) {
        Ok(true) => (),
        Ok(false) => {
            message::info(&format!("Enter a valid string "));
            return Ok(());
        }
        Err(e) => failure::bail!(e),
    }

    if target.account_id.is_empty() {
        failure::bail!(format!(
            "{} You must provide an account_id in your wrangler.toml before creating a secret!",
            emoji::WARN
        ))
    }
    put_secret(&name, &user, &target);
    bind_secret(&name, &user, &target)
    
}

