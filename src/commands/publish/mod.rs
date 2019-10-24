mod krate;
pub mod package;
pub mod preview;
mod route;
mod upload_form;

pub use package::Package;

use crate::settings::target::KvNamespace;
use route::Route;

use upload_form::build_script_and_upload_form;

use std::path::Path;

use crate::commands::kv;
use crate::commands::kv::bucket::AssetManifest;
use crate::commands::subdomain::Subdomain;
use crate::commands::validate_worker_name;
use crate::http;
use crate::settings::global_user::GlobalUser;

use crate::settings::target::{Site, Target};
use crate::terminal::{emoji, message};

pub fn publish(
    user: &GlobalUser,
    target: &mut Target,
    verbose: bool,
) -> Result<(), failure::Error> {
    let msg = match &target.route {
        Some(route) => &route,
        None => "workers_dev",
    };

    log::info!("{}", msg);

    validate_target_required_fields_present(target)?;
    validate_worker_name(&target.name)?;

    if let Some(site_config) = target.site.clone() {
        bind_static_site_contents(user, target, &site_config, false)?;
    }

    let asset_manifest = upload_buckets(target, user, verbose)?;
    build_and_publish_script(&user, &target, asset_manifest)?;

    Ok(())
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

fn build_and_publish_script(
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

    let script_upload_form = build_script_and_upload_form(target, asset_manifest)?;

    let mut res = client
        .put(&worker_addr)
        .multipart(script_upload_form)
        .send()?;

    let res_status = res.status();
    let res_text = res.text()?;

    if !res_status.is_success() {
        failure::bail!(error_msg(res_status, res_text))
    }

    let pattern = if target.route.is_some() {
        let route = Route::new(&target)?;
        Route::publish(&user, &target, &route)?;
        log::info!("publishing to route");
        route.pattern
    } else {
        log::info!("publishing to subdomain");
        publish_to_subdomain(target, user)?
    };

    log::info!("{}", &pattern);
    message::success(&format!(
        "Successfully published your script to {}",
        &pattern
    ));

    Ok(())
}

fn error_msg(status: reqwest::StatusCode, text: String) -> String {
    if text.contains("\"code\": 10034,") {
        "You need to verify your account's email address before you can publish. You can do this by checking your email or logging in to https://dash.cloudflare.com.".to_string()
    } else {
        format!("Something went wrong! Status: {}, Details {}", status, text)
    }
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

pub fn upload_buckets(
    target: &Target,
    user: &GlobalUser,
    verbose: bool,
) -> Result<Option<AssetManifest>, failure::Error> {
    let mut asset_manifest = None;
    for namespace in &target.kv_namespaces() {
        if let Some(bucket) = &namespace.bucket {
            if bucket.is_empty() {
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

fn publish_to_subdomain(target: &Target, user: &GlobalUser) -> Result<String, failure::Error> {
    log::info!("checking that subdomain is registered");
    let subdomain = Subdomain::get(&target.account_id, user)?;
    let subdomain = match subdomain {
        Some(subdomain) => subdomain,
        None => failure::bail!("Before publishing to workers.dev, you must register a subdomain. Please choose a name for your subdomain and run `wrangler subdomain <name>`.")
    };

    let sd_worker_addr = format!(
        "https://api.cloudflare.com/client/v4/accounts/{}/workers/scripts/{}/subdomain",
        target.account_id, target.name,
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
    Ok(format!("https://{}.{}.workers.dev", target.name, subdomain))
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

    let destination = if target.route.is_some() {
        // check required fields for publishing to a route
        if target
            .zone_id
            .as_ref()
            .unwrap_or(&"".to_string())
            .is_empty()
        {
            missing_fields.push("zone_id")
        };
        if target.route.as_ref().unwrap_or(&"".to_string()).is_empty() {
            missing_fields.push("route")
        };
        // zoned deploy destination
        "a route"
    } else {
        // zoneless deploy destination
        "your subdomain"
    };

    let (field_pluralization, is_are) = match missing_fields.len() {
        n if n >= 2 => ("fields", "are"),
        1 => ("field", "is"),
        _ => ("", ""),
    };

    if !missing_fields.is_empty() {
        failure::bail!(
            "{} Your wrangler.toml is missing the {} {:?} which {} required to publish to {}!",
            emoji::WARN,
            field_pluralization,
            missing_fields,
            is_are,
            destination
        );
    };

    Ok(())
}
