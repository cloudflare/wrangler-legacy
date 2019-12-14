use super::cli_prelude::*;

pub fn sub_command() -> App {
    SubCommand::with_name("whoami").about("{} Retrieve your user info and test your auth config")
}
