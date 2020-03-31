mod host;
mod session;
mod tunnel;

use host::Host;
use session::Session;
use tunnel::Tunnel;

use tokio;

use crate::settings::global_user::GlobalUser;
use crate::settings::toml::Target;

pub struct Tail;

impl Tail {
    pub async fn run(target: Target, user: GlobalUser) -> Result<(), failure::Error> {
        let res = tokio::try_join!(Host::run(), Tunnel::run(), Session::run(&target, &user));

        match res {
            Ok(_) => Ok(()),
            Err(e) => failure::bail!(e),
        }
    }
}
