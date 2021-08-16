use crate::commands::subdomain::Subdomain;
use crate::http;
use crate::settings::global_user::GlobalUser;
use crate::settings::toml::target::LazyAccountId;
use crate::settings::toml::RouteConfig;

use anyhow::Result;

#[derive(Clone, Debug, PartialEq)]
pub struct ZonelessTarget {
    pub account_id: LazyAccountId,
    pub script_name: String,
}

impl ZonelessTarget {
    pub fn build(script_name: &str, route_config: &RouteConfig) -> Result<Self> {
        Ok(Self {
            script_name: script_name.to_string(),
            account_id: route_config.account_id.clone(),
        })
    }

    pub fn deploy(&self, user: &GlobalUser) -> Result<String> {
        log::info!("publishing to workers.dev subdomain");
        log::info!("checking that subdomain is registered");
        let subdomain = match Subdomain::get(self.account_id.load()?, user)? {
            Some(subdomain) => subdomain,
            None => anyhow::bail!("Before publishing to workers.dev, you must register a subdomain. Please choose a name for your subdomain and run `wrangler subdomain <name>`.")
        };

        let sd_worker_addr = format!(
            "https://api.cloudflare.com/client/v4/accounts/{}/workers/scripts/{}/subdomain",
            self.account_id.load()?,
            self.script_name,
        );

        let client = http::legacy_auth_client(user);

        log::info!("Making public on subdomain...");
        let res = client
            .post(&sd_worker_addr)
            .header("Content-type", "application/json")
            .body(build_subdomain_request())
            .send()?;

        let status = res.status();
        let text = res.text()?;
        if !status.is_success() {
            anyhow::bail!(crate::format_api_errors(text))
        }

        let deploy_address = format!("https://{}.{}.workers.dev", self.script_name, subdomain);

        Ok(deploy_address)
    }
}

fn build_subdomain_request() -> String {
    serde_json::json!({ "enabled": true }).to_string()
}
