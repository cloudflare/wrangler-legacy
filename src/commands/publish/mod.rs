mod krate;
pub mod package;
mod route;
pub mod upload_form;

pub use package::Package;
use route::publish_routes;

use std::env;
use std::path::Path;

use crate::commands;
use crate::commands::kv;
use crate::commands::kv::bucket::AssetManifest;
use crate::commands::subdomain::Subdomain;
use crate::http;
use crate::settings::global_user::GlobalUser;
use crate::settings::toml::{DeployTarget, KvNamespace, Site, Target, Zoneless};
use crate::terminal::{emoji, message};

pub fn publish(
    user: &GlobalUser,
    target: &mut Target,
    deploy_target: DeployTarget,
    verbose: bool,
) -> Result<(), failure::Error> {
    validate_target_required_fields_present(target)?;

    // TODO: write a separate function for publishing a site
    if let Some(site_config) = &target.site.clone() {
        warn_site_incompatible_route(&deploy_target);
        bind_static_site_contents(user, target, &site_config, false)?;
    }

    let asset_manifest = upload_buckets(target, user, verbose)?;

    // Build the script before uploading.
    commands::build(&target)?;

    upload_script(&user, &target, asset_manifest)?;

    deploy(&user, &deploy_target)?;

    Ok(())
}

fn warn_site_incompatible_route(deploy_target: &DeployTarget) {
    if let DeployTarget::Zoned(zoned) = &deploy_target {
        if zoned.routes.len() == 1 {
            let route = &zoned.routes[0];

            if !route.pattern.ends_with('*') {
                message::warn(&format!(
                    "The route in your wrangler.toml should have a trailing * to apply the Worker on every path, otherwise your site will not behave as expected.\nroute = {}*",
                    route.pattern)
                );
            }
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

fn upload_script(
    user: &GlobalUser,
    target: &Target,
    asset_manifest: Option<AssetManifest>,
) -> Result<(), failure::Error> {
    let worker_addr = format!(
        "https://api.cloudflare.com/client/v4/accounts/{}/workers/scripts/{}",
        target.account_id, target.name,
    );

    let client = if target.site.is_some() {
        http::auth_client(Some("site"), user)
    } else {
        http::auth_client(None, user)
    };

    let script_upload_form = upload_form::build(target, asset_manifest)?;

    let mut res = client
        .put(&worker_addr)
        .multipart(script_upload_form)
        .send()?;

    let res_status = res.status();

    if !res_status.is_success() {
        let res_text = res.text()?;
        failure::bail!(error_msg(res_status, res_text))
    }

    Ok(())
}

fn deploy(user: &GlobalUser, deploy_target: &DeployTarget) -> Result<(), failure::Error> {
    match deploy_target {
        DeployTarget::Zoneless(zoneless_config) => {
            // this is a zoneless deploy
            log::info!("publishing to subdomain");
            let deploy_address = publish_to_subdomain(user, zoneless_config)?;

            message::success(&format!(
                "Successfully published your script to {}",
                deploy_address
            ));

            Ok(())
        }
        DeployTarget::Zoned(zoned_config) => {
            // this is a zoned deploy
            log::info!("publishing to zone {}", zoned_config.zone_id);

            let published_routes = publish_routes(&user, zoned_config)?;

            let display_results: Vec<String> =
                published_routes.iter().map(|r| format!("{}", r)).collect();

            message::success(&format!(
                "Deployed to the following routes:\n{}",
                display_results.join("\n")
            ));

            Ok(())
        }
    }
}

fn error_msg(status: reqwest::StatusCode, text: String) -> String {
    if text.contains("\"code\": 10034,") {
        "You need to verify your account's email address before you can publish. You can do this by checking your email or logging in to https://dash.cloudflare.com.".to_string()
    } else if text.contains("\"code\":10000,") {
        "Your user configuration is invalid, please run wrangler config and enter a new set of credentials.".to_string()
    } else {
        format!("Something went wrong! Status: {}, Details {}", status, text)
    }
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

fn build_subdomain_request() -> String {
    serde_json::json!({ "enabled": true }).to_string()
}

fn publish_to_subdomain(
    user: &GlobalUser,
    zoneless_config: &Zoneless,
) -> Result<String, failure::Error> {
    log::info!("checking that subdomain is registered");
    let subdomain = Subdomain::get(&zoneless_config.account_id, user)?;
    let subdomain = match subdomain {
        Some(subdomain) => subdomain,
        None => failure::bail!("Before publishing to workers.dev, you must register a subdomain. Please choose a name for your subdomain and run `wrangler subdomain <name>`.")
    };

    let sd_worker_addr = format!(
        "https://api.cloudflare.com/client/v4/accounts/{}/workers/scripts/{}/subdomain",
        zoneless_config.account_id, zoneless_config.script_name,
    );

    let client = http::auth_client(None, user);

    log::info!("Making public on subdomain...");
    let mut res = client
        .post(&sd_worker_addr)
        .header("Content-type", "application/json")
        .body(build_subdomain_request())
        .send()?;

    if !res.status().is_success() {
        failure::bail!(
            "Something went wrong! Status: {}, Details {}",
            res.status(),
            res.text()?
        )
    }

    Ok(format!(
        "https://{}.{}.workers.dev",
        zoneless_config.script_name, subdomain
    ))
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

#[test]
fn fails_with_good_error_msg_on_verify_email_err() {
    let status = reqwest::StatusCode::FORBIDDEN;
    let text = r#"{
  "result": null,
  "success": false,
  "errors": [
    {
      "code": 10034,
      "message": "workers.api.error.email_verification_required"
    }
  ],
  "messages": []
}"#
    .to_string();
    let result = error_msg(status, text);
    assert!(result.contains("https://dash.cloudflare.com"));
}
