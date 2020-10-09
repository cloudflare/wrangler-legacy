mod schedule;
mod zoned;
mod zoneless;

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

pub fn worker(
    user: &GlobalUser,
    deployment: &[DeployTarget],
) -> Result<Vec<String>, failure::Error> {
    let mut urls = Vec::new();
    for target in deployment {
        match target {
            DeployTarget::Zoned(zoned) => {
                let route_urls = zoned.deploy(user)?;
                urls.extend_from_slice(&route_urls);
            }
            DeployTarget::Zoneless(zoneless) => {
                let worker_dev = zoneless.deploy(user)?;
                urls.push(worker_dev);
            }
            DeployTarget::Schedule(schedule) => schedule.deploy(user)?,
        }
    }

    Ok(urls)
}
