use crate::login;
use anyhow::Result;

pub fn run(scopes_list: Option<&[&str]>) -> Result<()> {
    login::run(scopes_list)
}
