use std::process::Command;
use super::upload_and_get_id;
use crate::commands;
use uuid::Uuid;
use log::info;
use crate::http;
use crate::settings::{global_user::GlobalUser, project::Project};
use crate::terminal::message;
use failure::bail;
use shh;

use crate::process_builder::process;
use super::PREVIEW_ADDRESS;

pub fn httpie(
    project: Project,
    user: Option<GlobalUser>,
    preview_host: String,
    args: &[&str],
    ) -> Result<(), failure::Error> {
    commands::build(&project)?;

    let script_id = upload_and_get_id(&project, user.as_ref())?;

    let session = Uuid::new_v4().to_simple();
    let https = true;
    let https_str = if https { "https://" } else { "http://" };

    let cookie = format!(
        "__ew_fiddle_preview={}{}{}{}",
        script_id, session, https as u8, preview_host
    );

    process("http")
        .arg(PREVIEW_ADDRESS)
        .arg(format!("Cookie:{}", cookie))
        .exec_replace()
}

pub fn httpie_prompt(
    project: Project,
    user: Option<GlobalUser>,
    preview_host: String,
    ) -> Result<(), failure::Error> {
    commands::build(&project)?;

    let script_id = upload_and_get_id(&project, user.as_ref())?;

    let session = Uuid::new_v4().to_simple();
    let https = true;
    let https_str = if https { "https://" } else { "http://" };

    let cookie = format!(
        "__ew_fiddle_preview={}{}{}{}",
        script_id, session, https as u8, preview_host
    );

    process("http-prompt")
        .arg(PREVIEW_ADDRESS)
        .arg(format!("Cookie:{}", cookie))
        .exec_replace()
}
