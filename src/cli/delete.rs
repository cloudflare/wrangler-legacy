use crate::commands;

pub fn delete(no_interactive: bool, force: bool, account_id: Option<String>, script_id: Option<String>) -> Result<(), anyhow::Error> {

    commands::delete::run(no_interactive, force, account_id, script_id)
}
