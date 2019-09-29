use crate::http;
use crate::settings::global_user::GlobalUser;
use crate::settings::target::Target;
use crate::terminal::{emoji, message};

use serde::{Deserialize, Serialize};

#[derive(Serialize)]
pub struct Subdomain {
    subdomain: String,
}

impl Subdomain {
    fn request(account_id: &str, user: &GlobalUser) -> Result<Response, failure::Error> {
        let addr = subdomain_addr(account_id);

        let client = http::auth_client(None, user);

        let mut res = client.get(&addr).send()?;

        if !res.status().is_success() {
            failure::bail!(
                "{} There was an error fetching your subdomain.\n Status Code: {}\n Msg: {}",
                emoji::WARN,
                res.status(),
                res.text()?,
            )
        }

        Ok(serde_json::from_str(&res.text()?)?)
    }

    pub fn get(account_id: &str, user: &GlobalUser) -> Result<String, failure::Error> {
        let res = Subdomain::request(account_id, user)?;
        Ok(res
            .result
            .expect("Oops! We expected a subdomain name, but found none.")
            .subdomain)
    }

    pub fn get_as_option(
        account_id: &str,
        user: &GlobalUser,
    ) -> Result<Option<String>, failure::Error> {
        let res = Subdomain::request(account_id, user)?;
        Ok(res.result.map(|r| r.subdomain))
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

fn register_domain(name: &str, user: &GlobalUser, target: &Target) -> Result<(), failure::Error> {
    let msg = format!(
        "Registering your subdomain, {}.workers.dev, this could take up to a minute.",
        name
    );
    message::working(&msg);
    let account_id = &target.account_id;
    let addr = subdomain_addr(account_id);
    let sd = Subdomain {
        subdomain: name.to_string(),
    };
    let sd_request = serde_json::to_string(&sd)?;

    let client = http::auth_client(None, user);

    let mut res = client.put(&addr).body(sd_request).send()?;

    let msg;
    if !res.status().is_success() {
        let res_text = res.text()?;
        let res_json: Response = serde_json::from_str(&res_text)?;
        if already_has_subdomain(res_json.errors) {
            let sd = Subdomain::get(account_id, user)?;
            if sd == name {
                msg = format!(
                    "{} You have previously registered {}.workers.dev \n Status Code: {}\n Msg: {}",
                    emoji::WARN,
                    sd,
                    res.status(),
                    res_text,
                )
            } else {
                msg = format!(
                    "{} This account already has a registered subdomain. You can only register one subdomain per account. Your subdomain is {}.workers.dev \n Status Code: {}\n Msg: {}",
                    emoji::WARN,
                    sd,
                    res.status(),
                    res_text,
                )
            }
        } else if res.status() == 409 {
            msg = format!(
                "{} Your requested subdomain is not available. Please pick another one.\n Status Code: {}\n Msg: {}",
                emoji::WARN,
                res.status(),
                res_text
            );
        } else {
            msg = format!(
                "{} There was an error creating your requested subdomain.\n Status Code: {}\n Msg: {}",
                emoji::WARN,
                res.status(),
                res_text
            );
        }
        failure::bail!(msg)
    }
    let msg = format!("Success! You've registered {}.", name);
    message::success(&msg);
    Ok(())
}

pub fn subdomain(
    name: Option<&str>,
    user: &GlobalUser,
    target: &Target,
) -> Result<(), failure::Error> {
    if target.account_id.is_empty() {
        failure::bail!(format!(
            "{} You must provide an account_id in your wrangler.toml before creating a subdomain!",
            emoji::WARN
        ))
    }
    let subdomain = Subdomain::get_as_option(&target.account_id, user);
    return match (name, subdomain) {
        (None, Ok(None)) => {
            let msg = format!(
                "No subdomain registered. Use `wrangler subdomain <name>` to register one."
            );
            message::user_error(&msg);
            Ok(())
        }
        (None, Ok(Some(subdomain))) => {
            let msg = format!("{}.workers.dev", subdomain);
            message::info(&msg);
            Ok(())
        }
        (Some(name), Ok(None)) => register_domain(&name, &user, &target),
        (Some(_), Ok(Some(subdomain))) => {
            let msg = format!("This account already has a registered subdomain. You can only register one subdomain per account. Your subdomain is {}.workers.dev", subdomain);
            message::user_error(&msg);
            Ok(())
        }
        (_, Err(error)) => Err(error),
    };
}

fn already_has_subdomain(errors: Vec<Error>) -> bool {
    for error in errors {
        if error.code == 10036 {
            return true;
        }
    }
    false
}
