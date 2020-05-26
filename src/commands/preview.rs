use crate::preview;
use crate::preview::PreviewOpt;
use crate::settings::global_user::GlobalUser;
use crate::settings::toml::Target;

pub fn run(
    target: Target,
    user: Option<GlobalUser>,
    options: PreviewOpt,
    verbose: bool,
) -> Result<(), failure::Error> {
    preview(target, user, options, verbose)
}
