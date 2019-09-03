mod krate;
pub mod package;
pub mod preview;
mod route;
mod upload_form;

pub use package::Package;
use route::Route;
use upload_form::build_script_upload_form;

use log::info;

use crate::commands;
use crate::commands::subdomain::Subdomain;
use crate::http;
use crate::settings::global_user::GlobalUser;
use crate::settings::project::Target;
use crate::terminal::message;

pub fn publish(user: &GlobalUser, target: &Target, release: bool) -> Result<(), failure::Error> {
    info!("release = {}", release);

    if release {
        message::warn("--release will be deprecated, please use --environment or specify the workersdotdev boolean in the top of your `wrangler.toml`");
    }

    validate_target(target, release)?;
    commands::build(&target)?;
    publish_script(&user, &target, release)?;
    if release {
        info!("release mode detected, making a route...");
        let route = Route::new(&target)?;
        Route::publish(&user, &target, &route)?;
        let msg = format!(
            "Success! Your worker was successfully published. You can view it at {}.",
            &route.pattern
        );
        message::success(&msg);
    } else {
        message::success("Success! Your worker was successfully published.");
    }
    Ok(())
}

pub fn publish_environment(user: &GlobalUser, target: &Target) -> Result<(), failure::Error> {
    validate_target(target, true)?;
    commands::build(&target)?;
    publish_script(&user, &target, true)?;
    let route = Route::new(&target)?;
    Route::publish(&user, &target, &route)?;
    message::success(
        &format!(
            "Success! Your worker was successfully published. You can view it at {}.",
            &route.pattern
        )
        .to_owned(),
    );

    Ok(())
}

fn publish_script(user: &GlobalUser, target: &Target, release: bool) -> Result<(), failure::Error> {
    let worker_addr = format!(
        "https://api.cloudflare.com/client/v4/accounts/{}/workers/scripts/{}",
        target.account_id, target.name,
    );

    let client = http::auth_client(user);

    let script_upload_form = build_script_upload_form(target)?;

    let mut res = client
        .put(&worker_addr)
        .multipart(script_upload_form)
        .send()?;

    if res.status().is_success() {
        message::success("Successfully published your script.");
    } else {
        failure::bail!(
            "Something went wrong! Status: {}, Details {}",
            res.status(),
            res.text()?
        )
    }

    if !release {
        let private = target.private.unwrap_or(false);
        if !private {
            info!("--release not passed, publishing to subdomain");
            make_public_on_subdomain(target, user)?;
        }
    }

    Ok(())
}

fn build_subdomain_request() -> String {
    serde_json::json!({ "enabled": true }).to_string()
}

fn make_public_on_subdomain(target: &Target, user: &GlobalUser) -> Result<(), failure::Error> {
    info!("checking that subdomain is registered");
    let subdomain = Subdomain::get(&target.account_id, user)?;

    let sd_worker_addr = format!(
        "https://api.cloudflare.com/client/v4/accounts/{}/workers/scripts/{}/subdomain",
        target.account_id, target.name,
    );

    let client = http::auth_client(user);

    info!("Making public on subdomain...");
    let mut res = client
        .post(&sd_worker_addr)
        .header("Content-type", "application/json")
        .body(build_subdomain_request())
        .send()?;

    if res.status().is_success() {
        let msg = format!(
            "Successfully made your script available at https://{}.{}.workers.dev",
            target.name, subdomain
        );
        message::success(&msg)
    } else {
        failure::bail!(
            "Something went wrong! Status: {}, Details {}",
            res.status(),
            res.text()?
        )
    }
    Ok(())
}

fn validate_target(target: &Target, release: bool) -> Result<(), failure::Error> {
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

    let destination = if release {
        // check required fields for release
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
            "Your wrangler.toml is missing the {} {:?} which {} required to publish to {}!",
            field_pluralization,
            missing_fields,
            is_are,
            destination
        );
    };

    Ok(())
}
