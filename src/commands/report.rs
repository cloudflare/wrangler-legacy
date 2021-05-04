use crate::{
    http,
    reporter::{self, Report},
    settings,
};

use anyhow::Result;
use cloudflare::framework::{
    apiclient::ApiClient,
    endpoint::{Endpoint, Method},
    response::ApiFailure,
};

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

pub fn run(log: Option<&str>) -> Result<()> {
    let user = settings::global_user::GlobalUser::new()?;
    let report = reporter::read_log(log)?;
    let client = http::cf_v4_client(&user)?;
    client
        .request(&ErrorReport(report))
        .map(|_| ())
        .map_err(|e| match e {
            ApiFailure::Error(code, _) => {
                // a 409 Conflict will be returned if we have detected that a report has already
                // been submitted.
                if code == 409 {
                    return anyhow::anyhow!(
                        "we have already received this report, thank you for submitting!"
                    );
                }

                anyhow::format_err!(
                    "submission failed! please verify your credentials and try again. (status: {})",
                    code
                )
            }
            ApiFailure::Invalid(e) => {
                anyhow::format_err!("submission failed! please try again. (request error: {}", e)
            }
        })
}
