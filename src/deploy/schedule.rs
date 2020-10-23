use crate::http;
use crate::settings::global_user::GlobalUser;

#[derive(Clone, Debug, PartialEq)]
pub struct ScheduleTarget {
    pub account_id: String,
    pub script_name: String,
    pub crons: Vec<String>,
}

impl ScheduleTarget {
    pub fn build(
        account_id: String,
        script_name: String,
        crons: Vec<String>,
    ) -> Result<Self, failure::Error> {
        // TODO: add validation for expressions before pushing them to the API
        // we can do this once the cron parser is open sourced
        Ok(Self {
            account_id,
            script_name,
            crons,
        })
    }

    pub fn deploy(&self, user: &GlobalUser) -> Result<Vec<String>, failure::Error> {
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

        if !res.status().is_success() {
            failure::bail!(
                "Something went wrong! Status: {}, Details {}",
                res.status(),
                res.text()?
            )
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
