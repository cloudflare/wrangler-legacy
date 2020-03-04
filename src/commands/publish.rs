use std::env;
use std::path::Path;

use crate::commands;
use crate::commands::kv;
use crate::commands::kv::bucket::AssetManifest;
use crate::deploy;
use crate::settings::global_user::GlobalUser;
use crate::settings::toml::{DeployConfig, KvNamespace, Site, Target};
use crate::terminal::{emoji, message};
use crate::upload;

pub fn publish(
    user: &GlobalUser,
    target: &mut Target,
    deploy_config: DeployConfig,
    verbose: bool,
) -> Result<(), failure::Error> {
    validate_target_required_fields_present(target)?;

    // TODO: write a separate function for publishing a site
    if let Some(site_config) = &target.site.clone() {
        warn_site_incompatible_route(&deploy_config);
        bind_static_site_contents(user, target, &site_config, false)?;
    }

    let asset_manifest = upload_buckets(target, user, verbose)?;

    // Build the script before uploading.
    commands::build(&target)?;

    upload::script(&user, &target, asset_manifest)?;

    deploy::worker(&user, &deploy_config)?;

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

// Updates given Target with kv_namespace binding for a static site assets KV namespace.
pub fn bind_static_site_contents(
    user: &GlobalUser,
    target: &mut Target,
    site_config: &Site,
    preview: bool,
) -> Result<(), failure::Error> {
    let site_namespace = kv::namespace::site(target, &user, preview)?;

    // Check if namespace already is in namespace list
    for namespace in target.kv_namespaces() {
        if namespace.id == site_namespace.id {
            return Ok(()); // Sites binding already exists; ignore
        }
    }

    target.add_kv_namespace(KvNamespace {
        binding: "__STATIC_CONTENT".to_string(),
        id: site_namespace.id,
        bucket: Some(site_config.bucket.to_owned()),
    });
    Ok(())
}

pub fn upload_buckets(
    target: &Target,
    user: &GlobalUser,
    verbose: bool,
) -> Result<Option<AssetManifest>, failure::Error> {
    let mut asset_manifest = None;
    for namespace in &target.kv_namespaces() {
        if let Some(bucket) = &namespace.bucket {
            // We don't want folks setting their bucket to the top level directory,
            // which is where wrangler commands are always called from.
            let current_dir = env::current_dir()?;
            if bucket.as_os_str() == current_dir {
                failure::bail!(
                    "{} You need to specify a bucket directory in your wrangler.toml",
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
            let manifest_result = kv::bucket::sync(target, user, &namespace.id, path, verbose)?;
            if target.site.is_some() {
                if asset_manifest.is_none() {
                    asset_manifest = Some(manifest_result)
                } else {
                    // only site manifest should be returned
                    unreachable!()
                }
            }
        }
    }

    Ok(asset_manifest)
}

fn validate_target_required_fields_present(target: &Target) -> Result<(), failure::Error> {
    let mut missing_fields = Vec::new();

    if target.account_id.is_empty() {
        missing_fields.push("account_id")
    };
    if target.name.is_empty() {
        missing_fields.push("name")
    };

    match &target.kv_namespaces {
        Some(kv_namespaces) => {
            for kv in kv_namespaces {
                if kv.binding.is_empty() {
                    missing_fields.push("kv-namespace binding")
                }

                if kv.id.is_empty() {
                    missing_fields.push("kv-namespace id")
                }
            }
        }
        None => {}
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
