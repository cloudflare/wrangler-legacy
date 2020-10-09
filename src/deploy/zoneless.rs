use crate::commands::subdomain::Subdomain;
use crate::http;
use crate::settings::global_user::GlobalUser;
use crate::settings::toml::RouteConfig;
use crate::terminal::message::{Message, StdOut};

#[derive(Clone, Debug, PartialEq)]
pub struct ZonelessTarget {
    pub account_id: String,
    pub script_name: String,
}

impl ZonelessTarget {
    pub fn build(script_name: &str, route_config: &RouteConfig) -> Result<Self, failure::Error> {
        match route_config.account_id.as_ref() {
            // TODO: Deserialize empty strings to None; cannot do this for account id
            // yet without a large refactor.
            Some(account_id) if !account_id.is_empty() => Ok(Self {
                script_name: script_name.to_string(),
                account_id: account_id.to_string(),
            }),
            _ => failure::bail!("field `account_id` is required to deploy to workers.dev"),
        }
    }

    pub fn deploy(&self, user: &GlobalUser) -> Result<String, failure::Error> {
        log::info!("publishing to workers.dev subdomain");
        log::info!("checking that subdomain is registered");
        let subdomain = match Subdomain::get(&self.account_id, user)? {
            Some(subdomain) => subdomain,
            None => failure::bail!("Before publishing to workers.dev, you must register a subdomain. Please choose a name for your subdomain and run `wrangler subdomain <name>`.")
        };

        let sd_worker_addr = format!(
            "https://api.cloudflare.com/client/v4/accounts/{}/workers/scripts/{}/subdomain",
            self.account_id, self.script_name,
        );

        let client = http::legacy_auth_client(user);

        log::info!("Making public on subdomain...");
        let res = client
            .post(&sd_worker_addr)
            .header("Content-type", "application/json")
            .body(build_subdomain_request())
            .send()?;

        if !res.status().is_success() {
            failure::bail!(
                "Something went wrong! Status: {}, Details {}",
                res.status(),
                res.text()?
            )
        }

        let deploy_address = format!("https://{}.{}.workers.dev", self.script_name, subdomain);

        StdOut::success(&format!(
            "Successfully published your script to {}",
            deploy_address
        ));

        Ok(deploy_address)
    }
}

fn build_subdomain_request() -> String {
    serde_json::json!({ "enabled": true }).to_string()
}
