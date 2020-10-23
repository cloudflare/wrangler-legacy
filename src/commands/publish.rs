use std::env;
use std::path::{Path, PathBuf};

use indicatif::{ProgressBar, ProgressStyle};
use serde::{Deserialize, Serialize};

use crate::build::build_target;
use crate::deploy::{self, DeploymentSet};
use crate::http::{self, Feature};
use crate::kv::bulk;
use crate::settings::global_user::GlobalUser;
use crate::settings::toml::Target;
use crate::sites;
use crate::terminal::emoji;
use crate::terminal::message::{Message, Output, StdErr, StdOut};
use crate::upload;

#[derive(Serialize, Deserialize, Default)]
pub struct PublishOutput {
    pub success: bool,
    pub name: String,
    pub urls: Vec<String>,
    pub schedules: Vec<String>,
}

pub fn publish(
    user: &GlobalUser,
    target: &mut Target,
    deployments: DeploymentSet,
    out: Output,
) -> Result<(), failure::Error> {
    validate_target_required_fields_present(target)?;

    let deploy = |target: &Target| match deploy::worker(&user, &deployments) {
        Ok(deploy::DeployResults { urls, schedules }) => {
            let result_msg = match (urls.as_slice(), schedules.as_slice()) {
                ([], []) => "Successfully published your script".to_owned(),
                ([], schedules) => format!(
                    "Successfully published your script with this schedule\n {}",
                    schedules.join("\n ")
                ),
                (urls, []) => format!(
                    "Successfully published your script to\n {}",
                    urls.join("\n ")
                ),
                (urls, schedules) => format!(
                    "Successfully published your script to\n {}\nwith this schedule\n {}",
                    urls.join("\n "),
                    schedules.join("\n ")
                ),
            };
            StdErr::success(&result_msg);
            if out == Output::Json {
                StdOut::as_json(&PublishOutput {
                    success: true,
                    name: target.name.clone(),
                    urls,
                    schedules,
                });
            }
            Ok(())
        }
        Err(e) => Err(e),
    };

    // Build the script before uploading and log build result
    let build_result = build_target(&target);
    match build_result {
        Ok(msg) => {
            StdErr::success(&msg);
            Ok(())
        }
        Err(e) => Err(e),
    }?;
    if let Some(site_config) = &target.site {
        let path = &site_config.bucket.clone();
        validate_bucket_location(path)?;

        let site_namespace = sites::add_namespace(user, target, false)?;

        let (to_upload, to_delete, asset_manifest) =
            sites::sync(target, user, &site_namespace.id, &path)?;

        // First, upload all existing files in bucket directory
        StdErr::working("Uploading site files");
        let upload_progress_bar = if to_upload.len() > bulk::BATCH_KEY_MAX {
            let upload_progress_bar = ProgressBar::new(to_upload.len() as u64);
            upload_progress_bar
                .set_style(ProgressStyle::default_bar().template("{wide_bar} {pos}/{len}\n{msg}"));
            Some(upload_progress_bar)
        } else {
            None
        };

        bulk::put(
            target,
            user,
            &site_namespace.id,
            to_upload,
            &upload_progress_bar,
        )?;

        if let Some(pb) = upload_progress_bar {
            pb.finish_with_message("Done Uploading");
        }

        let upload_client = http::featured_legacy_auth_client(user, Feature::Sites);

        // Next, upload and deploy the worker with the updated asset_manifest
        upload::script(&upload_client, &target, Some(asset_manifest))?;

        deploy(target)?;

        // Finally, remove any stale files
        if !to_delete.is_empty() {
            StdErr::info("Deleting stale files...");

            let delete_progress_bar = if to_delete.len() > bulk::BATCH_KEY_MAX {
                let delete_progress_bar = ProgressBar::new(to_delete.len() as u64);
                delete_progress_bar.set_style(
                    ProgressStyle::default_bar().template("{wide_bar} {pos}/{len}\n{msg}"),
                );
                Some(delete_progress_bar)
            } else {
                None
            };

            bulk::delete(
                target,
                user,
                &site_namespace.id,
                to_delete,
                &delete_progress_bar,
            )?;

            if let Some(pb) = delete_progress_bar {
                pb.finish_with_message("Done deleting");
            }
        }
    } else {
        let upload_client = http::legacy_auth_client(user);

        upload::script(&upload_client, &target, None)?;
        deploy(target)?;
    }

    Ok(())
}

// We don't want folks setting their bucket to the top level directory,
// which is where wrangler commands are always called from.
pub fn validate_bucket_location(bucket: &PathBuf) -> Result<(), failure::Error> {
    // TODO: this should really use a convenience function for "Wrangler Project Root"
    let current_dir = env::current_dir()?;
    if bucket.as_os_str() == current_dir {
        failure::bail!(
            "{} Your bucket cannot be set to the parent directory of your configuration file",
            emoji::WARN
        )
    }
    let path = Path::new(&bucket);
    if !path.exists() {
        failure::bail!(
            "{} bucket directory \"{}\" does not exist",
            emoji::WARN,
            path.display()
        )
    } else if !path.is_dir() {
        failure::bail!(
            "{} bucket \"{}\" is not a directory",
            emoji::WARN,
            path.display()
        )
    }

    Ok(())
}

fn validate_target_required_fields_present(target: &Target) -> Result<(), failure::Error> {
    let mut missing_fields = Vec::new();

    if target.account_id.is_empty() {
        missing_fields.push("account_id")
    };
    if target.name.is_empty() {
        missing_fields.push("name")
    };

    for kv in &target.kv_namespaces {
        if kv.binding.is_empty() {
            missing_fields.push("kv-namespace binding")
        }

        if kv.id.is_empty() {
            missing_fields.push("kv-namespace id")
        }
    }

    let (field_pluralization, is_are) = match missing_fields.len() {
        n if n >= 2 => ("fields", "are"),
        1 => ("field", "is"),
        _ => ("", ""),
    };

    if !missing_fields.is_empty() {
        failure::bail!(
            "{} Your configuration file is missing the {} {:?} which {} required to publish your worker!",
            emoji::WARN,
            field_pluralization,
            missing_fields,
            is_are,
        );
    };

    Ok(())
}
