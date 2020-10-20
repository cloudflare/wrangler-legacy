use crate::settings::global_user::GlobalUser;
use crate::settings::toml::{DeployConfig, Target};

pub fn upload_worker_form(
    user: &GlobalUser,
    target: &mut Target,
    deploy_config: DeployConfig,
    out: Output,
) -> Result<(), failure::Error> {
    validate_target_required_fields_present(target)?;
    //let upload_client = http::legacy_auth_client(user);
    let upload_client = http::cf_v4_api_client_async(user, HttpApiClientConfig::default());
    upload::script(&upload_client, &target, None)?;
    deploy::worker(&user, &deploy_config)?;
    Ok(())
}