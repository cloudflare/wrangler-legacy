use crate::preview::{preview, PreviewOpt};
use crate::settings::global_user::GlobalUser;
use crate::settings::toml::Target;

use anyhow::Result;

pub fn run(
    target: Target,
    user: Option<GlobalUser>,
    options: PreviewOpt,
    verbose: bool,
) -> Result<()> {
    preview(target, user, options, verbose)
}
