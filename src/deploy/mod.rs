mod durable_objects;
mod schedule;
mod zoned;
mod zoneless;

pub use durable_objects::DurableObjectsTarget;
pub use schedule::ScheduleTarget;
pub use zoned::ZonedTarget;
pub use zoneless::ZonelessTarget;

use crate::settings::{global_user::GlobalUser, toml::Target};

/// A set of deploy targets.
pub type DeploymentSet = Vec<DeployTarget>;

#[derive(Debug, PartialEq, Clone)]
pub enum DeployTarget {
    Zoned(ZonedTarget),
    Zoneless(ZonelessTarget),
    Schedule(ScheduleTarget),
    DurableObjects(DurableObjectsTarget),
}

pub fn pre_upload(
    user: &GlobalUser,
    target: &mut Target,
    deploy_targets: &[DeployTarget],
    only_hydrate: bool,
) -> Result<(), failure::Error> {
    for deploy_target in deploy_targets {
        if let DeployTarget::DurableObjects(durable_objects) = deploy_target {
            if only_hydrate {
                durable_objects.only_hydrate(user, target)?;
            } else {
                durable_objects.pre_upload(user, target)?;
            }
        }
    }

    Ok(())
}

pub fn deploy(
    user: &GlobalUser,
    deploy_targets: &[DeployTarget],
) -> Result<DeployResults, failure::Error> {
    let mut results = DeployResults::default();
    for target in deploy_targets {
        match target {
            DeployTarget::Zoned(zoned) => {
                let route_urls = zoned.deploy(user)?;
                results.urls.extend(route_urls);
            }
            DeployTarget::Zoneless(zoneless) => {
                let worker_dev = zoneless.deploy(user)?;
                results.urls.push(worker_dev);
            }
            DeployTarget::Schedule(schedule) => {
                let schedules = schedule.deploy(user)?;
                results.schedules.extend(schedules);
            }
            DeployTarget::DurableObjects(durable_objects) => {
                let namespaces = durable_objects.deploy(user)?;
                results.durable_object_namespaces.extend(namespaces);
            }
        }
    }

    Ok(results)
}

#[derive(Default)]
pub struct DeployResults {
    pub urls: Vec<String>,
    pub schedules: Vec<String>,
    pub durable_object_namespaces: Vec<String>,
}
