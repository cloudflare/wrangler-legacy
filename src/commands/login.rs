use crate::login;
use anyhow::Result;

pub fn run(scopes_list: Option<&[String]>) -> Result<()> {
    login::run(scopes_list)
}
