use crate::settings::global_user::GlobalUser;
use crate::settings::toml::Target;
use crate::tail::Tail;

pub fn start(target: &Target, user: &GlobalUser) -> Result<(), failure::Error> {
    Tail::run(target.clone(), user.clone())
}
