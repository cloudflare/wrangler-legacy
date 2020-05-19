use crate::preview;
use crate::settings::global_user::GlobalUser;
use crate::settings::toml::Target;

pub fn run(
    target: Target,
    user: Option<GlobalUser>,
    method: &str,
    url: &str,
    body: Option<String>,
    livereload: bool,
    verbose: bool,
    headless: bool,
) -> Result<(), failure::Error> {
    preview(
        target, user, method, url, body, livereload, verbose, headless,
    )
}
