use crate::http;
use crate::settings::global_user::GlobalUser;

use anyhow::Result;

#[derive(Clone, Debug, PartialEq)]
pub struct ScheduleTarget {
    pub account_id: String,
    pub script_name: String,
    pub crons: Vec<String>,
}

impl ScheduleTarget {
    pub fn deploy(&self, user: &GlobalUser) -> Result<Vec<String>> {
        log::info!("publishing schedules");
        let schedule_worker_addr = format!(
            "https://api.cloudflare.com/client/v4/accounts/{}/workers/scripts/{}/schedules",
            self.account_id, self.script_name,
        );

        let client = http::legacy_auth_client(user);

        log::info!("Pushing {} schedule(s)...", self.crons.len());
        let res = client
            .put(&schedule_worker_addr)
            .header("Content-Type", "application/json")
            .body(build_schedules_request(&self.crons))
            .send()?;

        let status = res.status();
        let text = res.text()?;
        if !status.is_success() {
            anyhow::bail!(crate::format_api_errors(text))
        }

        Ok(self.crons.clone())
    }
}

fn build_schedules_request(crons: &[String]) -> String {
    let values = crons
        .iter()
        .map(|s| serde_json::json!({ "cron": s }))
        .collect();
    serde_json::Value::Array(values).to_string()
}
