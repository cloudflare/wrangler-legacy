use crate::commands::dev::ServerConfig;
use crate::settings::global_user::GlobalUser;
use crate::settings::toml::Target;

pub fn dev(
    target: Target,
    user: GlobalUser,
    server_config: ServerConfig,
    verbose: bool,
) -> Result<(), failure::Error> {
    println!(
        "{:?}\n{:?}\n{:?}\n{:?}",
        target, user, server_config, verbose
    );
    Ok(())
}
