mod host;
mod session;
mod tunnel;

use host::Host;
use session::Session;
use tunnel::Tunnel;

use tokio;
use tokio::runtime::Runtime as TokioRuntime;

use crate::settings::global_user::GlobalUser;
use crate::settings::toml::Target;

pub struct Tail;

impl Tail {
    pub fn run(target: Target, user: GlobalUser) -> Result<(), failure::Error> {
        let mut runtime = TokioRuntime::new()?;

        runtime.block_on(async {
            let tunnel_process = Tunnel::new()?;
            let res = tokio::try_join!(
                Host::run(),
                tunnel_process.run(),
                Session::run(&target, &user)
            );

            match res {
                Ok(_) => Ok(()),
                Err(e) => failure::bail!(e),
            }
        })
    }
}
