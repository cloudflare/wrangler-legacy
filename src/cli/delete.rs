use crate::commands;
use crate::settings::global_user::GlobalUser;

pub fn delete(
    no_interactive: bool,
    force: bool,
    account_id: Option<String>,
    script_name: Option<String>,
) -> Result<(), anyhow::Error> {
    let user = GlobalUser::new()?;
    if no_interactive {
        if let (Some(account_id), Some(script_name)) = (account_id, script_name) {
            commands::delete::delete_script(&user, force, &account_id, &script_name)
        } else {
            anyhow::bail!("Both account-id and script-name must be provided.")
        }
    } else {
        commands::delete::run(&user)
    }
}
