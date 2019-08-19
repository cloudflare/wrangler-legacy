// todo(gabbi): This file should use cloudflare-rs instead of our http::auth_client 
// when https://github.com/cloudflare/cloudflare-rs/issues/26 is handled (this is 
// because the GET key operation doesn't return json on success--just the raw
// value).

use cloudflare::framework::response::ApiFailure;

use crate::settings::global_user::GlobalUser;
use crate::settings::project::Project;
use crate::http;

pub fn get_key(project: &Project, user: &GlobalUser, id: &str, key: &str) -> Result<(), failure::Error> {
    let api_endpoint = format!(
        "https://api.cloudflare.com/client/v4/accounts/{}/storage/kv/namespaces/{}/values/{}",
        project.account_id, id, key
    );

    let client = http::auth_client(user); 

    let mut res = client
    .get(&api_endpoint)
    .send()?;

    if res.status().is_success() {
        let body_text = res.text()?;
        // We don't use message::success because we don't want to include the emoji/formatting
        // in case someone is piping this to stdin
        println!("{}", &body_text);
    } else {
        // This is logic pulled from cloudflare-rs for pretty error formatting right now; 
        // it will be redundant when we switch to using cloudflare-rs for all API requests.
        let parsed = res.json();
        let errors = parsed.unwrap_or_default();
        super::print_error(ApiFailure::Error(res.status(), errors));
    }

    Ok(())
}