// TODO:(gabbi) This file should use cloudflare-rs instead of our http::auth_client
// when https://github.com/cloudflare/cloudflare-rs/issues/26 is handled (this is
// because the GET key operation doesn't return json on success--just the raw
// value).

use cloudflare::framework::response::ApiFailure;

use crate::commands::kv;
use crate::http;
use crate::settings::global_user::GlobalUser;
use crate::settings::toml::Target;

pub fn get(target: &Target, user: &GlobalUser, id: &str, key: &str) -> Result<(), failure::Error> {
    kv::validate_target(target)?;
    let api_endpoint = format!(
        "https://api.cloudflare.com/client/v4/accounts/{}/storage/kv/namespaces/{}/values/{}",
        target.account_id,
        id,
        kv::url_encode_key(key)
    );

    let client = http::auth_client(None, &user);

    let res = client.get(&api_endpoint).send()?;

    let response_status = res.status();
    if response_status.is_success() {
        let body_text = res.text()?;
        // We don't use message::success because we don't want to include the emoji/formatting
        // in case someone is piping this to stdin
        print!("{}", &body_text);
    } else {
        // This is logic pulled from cloudflare-rs for pretty error formatting right now;
        // it will be redundant when we switch to using cloudflare-rs for all API requests.
        let parsed = res.json();
        let errors = parsed.unwrap_or_default();
        print!(
            "{}",
            kv::format_error(ApiFailure::Error(response_status, errors))
        );
    }

    Ok(())
}
