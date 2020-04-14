use crate::settings::global_user::GlobalUser;
use crate::settings::toml::Target;
use crate::tail::Tail;

pub fn start(target: &Target, user: &GlobalUser) -> Result<(), failure::Error> {
    // Note that we use eprintln!() throughout this file; this is because we want any
    // helpful output to not be mixed with actual log JSON output, so we use this macro
    // to print messages to stderr instead of stdout (where log output is printed).
    eprintln!(
        "Setting up log streaming from Worker script \"{}\" to Wrangler. This may take a few seconds...",
        target.name
    );

    Tail::run(target.clone(), user.clone())
}
