// TODO:(gabbi) This file should use cloudflare-rs instead of our http::legacy_auth_client
// when https://github.com/cloudflare/cloudflare-rs/issues/26 is handled (this is
// because the GET key operation doesn't return json on success--just the raw
// value).

use cloudflare::framework::response::ApiFailure;

use anyhow::Result;

use crate::commands::kv;
use crate::http;
use crate::settings::global_user::GlobalUser;
use crate::settings::toml::Target;
use std::io::{self, Write};

pub fn get(target: &Target, user: &GlobalUser, id: &str, key: &str) -> Result<()> {
    let api_endpoint = format!(
        "https://api.cloudflare.com/client/v4/accounts/{}/storage/kv/namespaces/{}/values/{}",
        target.account_id.load()?,
        id,
        kv::url_encode_key(key)
    );

    let client = http::legacy_auth_client(user);

    let res = client.get(&api_endpoint).send()?;

    let response_status = res.status();
    if response_status.is_success() {
        let body = res.bytes()?;
        // We don't use message::success because we don't want to include the emoji/formatting
        // in case someone is piping this to stdin.
        // This will probably fail for non-UTF8 on Windows, but should at least work for people
        // getting binary data from KV on Unix-y systems.
        io::stdout().write_all(&*body)?;
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
