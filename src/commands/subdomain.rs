use crate::emoji;
use crate::http;
use crate::settings::global_user::GlobalUser;
use crate::settings::project::Project;

use serde::{Deserialize, Serialize};

#[derive(Serialize)]
pub struct Subdomain {
    subdomain: String,
}

impl Subdomain {
    pub fn get(account_id: &str, user: &GlobalUser) -> Result<String, failure::Error> {
        let addr = subdomain_addr(account_id);

        let client = http::client();

        let mut res = client
            .get(&addr)
            .header("X-Auth-Key", &*user.api_key)
            .header("X-Auth-Email", &*user.email)
            .send()?;

        if !res.status().is_success() {
            failure::bail!(
                "⛔ There was an error fetching your subdomain.\n Status Code: {}\n Msg: {}",
                res.status(),
                res.text()?,
            )
        }

        let res: Response = serde_json::from_str(&res.text()?)?;
        Ok(res
            .result
            .expect("Oops! We expected a subdomain name, but found none.")
            .subdomain)
    }
}

#[derive(Deserialize)]
struct Response {
    errors: Vec<Error>,
    result: Option<SubdomainResult>,
}

#[derive(Deserialize)]
struct Error {
    code: u32,
}

#[derive(Deserialize)]
struct SubdomainResult {
    subdomain: String,
}

fn subdomain_addr(account_id: &str) -> String {
    format!(
        "https://api.cloudflare.com/client/v4/accounts/{}/workers/subdomain",
        account_id
    )
}

pub fn subdomain(name: &str, user: &GlobalUser, project: &Project) -> Result<(), failure::Error> {
    println!(
        "{} Registering your subdomain, {}.workers.dev, this could take up to a minute.",
        emoji::SNAIL,
        name
    );
    let account_id = &project.account_id;
    let addr = subdomain_addr(account_id);
    let sd = Subdomain {
        subdomain: name.to_string(),
    };
    let sd_request = serde_json::to_string(&sd)?;

    let client = http::client();

    let mut res = client
        .put(&addr)
        .header("X-Auth-Key", &*user.api_key)
        .header("X-Auth-Email", &*user.email)
        .body(sd_request)
        .send()?;

    let msg;
    if !res.status().is_success() {
        let res_text = res.text()?;
        let res_json: Response = serde_json::from_str(&res_text)?;
        if already_has_subdomain(res_json.errors) {
            let sd = Subdomain::get(account_id, user)?;
            msg = format!(
                "⛔ This account already has a registered subdomain. You can only register one subdomain per account. Your subdomain is {}.workers.dev \n Status Code: {}\n Msg: {}",
                sd,
                res.status(),
                res_text,
            );
        } else if res.status() == 409 {
            msg = format!(
                "⛔ Your requested subdomain is not available. Please pick another one.\n Status Code: {}\n Msg: {}",
                res.status(),
                res_text
            );
        } else {
            msg = format!(
                "⛔ There was an error creating your requested subdomain.\n Status Code: {}\n Msg: {}",
                res.status(),
                res_text
            );
        }
        failure::bail!(msg)
    }
    println!("{} Success! You've registered {}.", emoji::SPARKLES, name);
    Ok(())
}

fn already_has_subdomain(errors: Vec<Error>) -> bool {
    for error in errors {
        if error.code == 10036 {
            return true;
        }
    }
    false
}
