use crate::preview::{preview, PreviewOpt};
use crate::settings::global_user::GlobalUser;
use crate::settings::toml::Target;

pub fn run(
    target: Target,
    user: Option<GlobalUser>,
    options: PreviewOpt,
    verbose: bool,
    build_env: Option<String>,
) -> Result<(), failure::Error> {
    preview(target, user, options, verbose, build_env)
}
