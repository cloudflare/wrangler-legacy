use crate::commands;

pub fn logout() -> Result<(), anyhow::Error> {
    commands::logout::run()
}
