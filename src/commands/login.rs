use crate::login;
use anyhow::Result;

pub fn run(scopes_list: Option<&Vec<String>>) -> Result<()> {
    login::run(scopes_list)
}
