mod schedule;
mod zoned;
mod zoneless;

use anyhow::Result;
use indicatif::{ProgressBar, ProgressStyle};
pub use schedule::ScheduleTarget;
pub use zoned::ZonedTarget;
pub use zoneless::ZonelessTarget;

use crate::settings::global_user::GlobalUser;

/// A set of deploy targets.
pub type DeploymentSet = Vec<DeployTarget>;

#[derive(Debug, PartialEq, Clone)]
pub enum DeployTarget {
    Zoned(ZonedTarget),
    Zoneless(ZonelessTarget),
    Schedule(ScheduleTarget),
}

pub fn deploy(user: &GlobalUser, deploy_targets: &[DeployTarget]) -> Result<DeployResults> {
    let style = ProgressStyle::default_spinner().template("{spinner}   {msg}");
    let spinner = ProgressBar::new_spinner().with_style(style);
    spinner.enable_steady_tick(20);
    let mut results = DeployResults::default();
    for target in deploy_targets {
        match target {
            DeployTarget::Zoned(zoned) => {
                spinner.set_message("Configuring routes...");
                let route_urls = zoned.deploy(user)?;
                results.urls.extend(route_urls);
            }
            DeployTarget::Zoneless(zoneless) => {
                spinner.set_message("Configuring workers.dev...");
                let worker_dev = zoneless.deploy(user)?;
                results.urls.push(worker_dev);
            }
            DeployTarget::Schedule(schedule) => {
                spinner.set_message("Configuring schedules...");
                let schedules = schedule.deploy(user)?;
                results.schedules.extend(schedules);
            }
        }
    }

    spinner.finish_and_clear();

    Ok(results)
}

#[derive(Default)]
pub struct DeployResults {
    pub urls: Vec<String>,
    pub schedules: Vec<String>,
}
