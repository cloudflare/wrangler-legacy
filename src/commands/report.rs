use crate::{
    http,
    reporter::{self, Report},
    settings,
};

use anyhow::{anyhow, Result};
use cloudflare::framework::{
    apiclient::ApiClient,
    endpoint::{Endpoint, Method},
    response::ApiFailure,
};
use std::path::Path;

struct ErrorReport(Report);

impl Endpoint<(), (), Report> for ErrorReport {
    fn method(&self) -> Method {
        Method::Post
    }

    fn path(&self) -> String {
        "wrangler/errors".into()
    }

    fn body(&self) -> Option<Report> {
        Some(self.0.clone())
    }
}

pub fn run(log: Option<&Path>) -> Result<()> {
    let user = settings::global_user::GlobalUser::new()?;
    let report = reporter::read_log(log)?;
    let client = http::cf_v4_client(&user)?;
    if let Err(e) = client.request(&ErrorReport(report)) {
        match e {
            ApiFailure::Error(code, _) => {
                // a 409 Conflict will be returned if we have detected that a report has already
                // been submitted.
                if code == 409 {
                    return Err(anyhow!(
                        "we already received this report, thank you for submitting!"
                    ));
                }

                // TODO: consider adding anyhow::Context to capture some more insightful detail here
                Err(anyhow!(
                    "submission failed! please verify your credentials and try again. (status: {})",
                    code
                ))
            }
            ApiFailure::Invalid(e) => Err(anyhow!(
                "submission failed! please try again. (request error: {})",
                e
            )),
        }
    } else {
        Ok(())
    }
}
