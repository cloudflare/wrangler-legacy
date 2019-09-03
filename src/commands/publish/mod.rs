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

pub fn publish(user: &GlobalUser, target: &Target) -> Result<(), failure::Error> {
    info!("workers_dot_dev = {}", target.workers_dot_dev);

    validate_target(target)?;
    commands::build(&target)?;
    publish_script(&user, &target)?;
    Ok(())
}

fn publish_script(user: &GlobalUser, target: &Target) -> Result<(), failure::Error> {
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

    let pattern = if !target.workers_dot_dev {
        let route = Route::new(&target)?;
        Route::publish(&user, &target, &route)?;
        info!("publishing to route");
        route.pattern
    } else {
        info!("publishing to subdomain");
        publish_to_subdomain(target, user)?
    };
    info!("{}", &pattern);
    message::success(&format!(
        "Success! Your worker was successfully published. You can view it at {}.",
        &pattern
    ));

    Ok(())
}

fn build_subdomain_request() -> String {
    serde_json::json!({ "enabled": true }).to_string()
}

fn publish_to_subdomain(target: &Target, user: &GlobalUser) -> Result<String, failure::Error> {
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

    if !res.status().is_success() {
        failure::bail!(
            "Something went wrong! Status: {}, Details {}",
            res.status(),
            res.text()?
        )
    }
    Ok(format!("https://{}.{}.workers.dev", target.name, subdomain))
}

fn validate_target(target: &Target) -> Result<(), failure::Error> {
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

    let destination = if !target.workers_dot_dev {
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
