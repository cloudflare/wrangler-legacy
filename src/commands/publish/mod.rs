use std::env;
use std::path::{Path, PathBuf};

use console::style;

use crate::commands;
use crate::commands::kv;
use crate::commands::kv::bucket::{sync, upload_files};
use crate::commands::kv::bulk::delete::delete_bulk;
use crate::deploy;
use crate::http::{self, Feature};
use crate::settings::global_user::GlobalUser;
use crate::settings::toml::{DeployConfig, Target};
use crate::terminal::{emoji, message};
use crate::upload;

pub fn publish(
    user: &GlobalUser,
    target: &mut Target,
    deploy_config: DeployConfig,
    verbose: bool,
) -> Result<(), failure::Error> {
    validate_target_required_fields_present(user, target)?;

    // Build the script before uploading.
    commands::build(&target)?;

    if let Some(site_config) = &target.site {
        let path = &site_config.bucket.clone();
        validate_bucket_location(path)?;
        warn_site_incompatible_route(&deploy_config);

        let site_namespace = site_config.kv_namespace(user, target)?;

        let (to_upload, to_delete, asset_manifest) = sync(target, user, &site_namespace.id, &path)?;

        // First, upload all existing files in bucket directory
        if verbose {
            message::info("Preparing to upload updated files...");
        }
        upload_files(target, user, &site_namespace.id, to_upload)?;

        let upload_client = http::featured_legacy_auth_client(user, Feature::Sites);

        // Next, upload and deploy the worker with the updated asset_manifest
        upload::script(&upload_client, user, &target, Some(asset_manifest))?;

        deploy::worker(&user, &deploy_config)?;

        // Finally, remove any stale files
        if !to_delete.is_empty() {
            if verbose {
                message::info("Deleting stale files...");
            }

            delete_bulk(target, user, &site_namespace.id, to_delete)?;
        }
    } else {
        let uses_kv_bucket = sync_non_site_buckets(target, user, verbose)?;

        let upload_client = if uses_kv_bucket {
            let wrangler_toml = style("`wrangler.toml`").yellow().bold();
            let issue_link = style("https://github.com/cloudflare/wrangler/issues/1136")
                .blue()
                .bold();
            let msg = format!("As of 1.9.0, you will no longer be able to specify a bucket for a kv namespace in your {}.\nIf your application depends on this feature, please file an issue with your use case here:\n{}", wrangler_toml, issue_link);
            message::deprecation_warning(&msg);

            http::featured_legacy_auth_client(user, Feature::Bucket)
        } else {
            http::legacy_auth_client(user)
        };

        upload::script(&upload_client, user, &target, None)?;

        deploy::worker(&user, &deploy_config)?;
    }

    Ok(())
}

// This checks all of the configured routes for the wildcard ending and warns
// the user that their site may not work as expected without it.
fn warn_site_incompatible_route(deploy_config: &DeployConfig) {
    if let DeployConfig::Zoned(zoned) = &deploy_config {
        let mut no_star_routes = Vec::new();
        for route in &zoned.routes {
            if !route.pattern.ends_with('*') {
                no_star_routes.push(route.pattern.to_string());
            }
        }

        if !no_star_routes.is_empty() {
            message::warn(&format!(
                "The following routes in your wrangler.toml should have a trailing * to apply the Worker on every path, otherwise your site will not behave as expected.\n{}",
                no_star_routes.join("\n"))
            );
        }
    }
}

// We don't want folks setting their bucket to the top level directory,
// which is where wrangler commands are always called from.
pub fn validate_bucket_location(bucket: &PathBuf) -> Result<(), failure::Error> {
    // TODO: this should really use a convenience function for "Wrangler Project Root"
    let current_dir = env::current_dir()?;
    if bucket.as_os_str() == current_dir {
        failure::bail!(
            "{} Your bucket cannot be set to the parent directory of your wrangler.toml",
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

// This is broken into a separate step because the intended design does not
// necessarily intend for bucket support outside of the [site] usage, especially
// since assets are still hashed. In a subsequent release, we will either
// deprecate this step, or we will integrate it more closely and adapt to user
// feedback.
//
// In order to track usage of this "feature", this function returns a bool that
// indicates whether any non-site kv namespaces were specified / uploaded.
pub fn sync_non_site_buckets(
    target: &Target,
    user: &GlobalUser,
    verbose: bool,
) -> Result<bool, failure::Error> {
    let mut is_using_non_site_bucket = false;

    for namespace in target.kv_namespaces(user, &mut target)? {
        if let Some(path) = &namespace.bucket {
            is_using_non_site_bucket = true;
            validate_bucket_location(path)?;
            let (to_upload, to_delete, _) = kv::bucket::sync(target, user, &namespace.id, path)?;
            // First, upload all existing files in bucket directory
            if verbose {
                message::info("Preparing to upload updated files...");
            }
            upload_files(target, user, &namespace.id, to_upload)?;

            // Finally, remove any stale files
            if !to_delete.is_empty() {
                if verbose {
                    message::info("Deleting stale files...");
                }

                delete_bulk(target, user, &namespace.id, to_delete)?;
            }
        }
    }

    Ok(is_using_non_site_bucket)
}

fn validate_target_required_fields_present(
    user: &GlobalUser,
    target: &Target,
) -> Result<(), failure::Error> {
    let mut missing_fields = Vec::new();

    if target.account_id.is_empty() {
        missing_fields.push("account_id")
    };
    if target.name.is_empty() {
        missing_fields.push("name")
    };

    for kv in target.kv_namespaces(user, &mut target)? {
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
            "{} Your wrangler.toml is missing the {} {:?} which {} required to publish your worker!",
            emoji::WARN,
            field_pluralization,
            missing_fields,
            is_are,
        );
    };

    Ok(())
}
