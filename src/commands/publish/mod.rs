mod krate;
pub mod preview;
mod route;
use route::Route;

use reqwest::multipart::Form;
use std::fs;
use std::path::Path;

use crate::user::User;
use crate::wranglerjs::Bundle;

pub fn publish(user: User, name: Option<&str>) -> Result<(), failure::Error> {
    if user.account.multiscript {
        if name.is_none() {
            println!("⚠️ You have multiscript account. Using a default name, 'wasm-worker'.")
        }
        let name = name.unwrap_or("wasm-worker");
        multi_script(&user, name)?;
        Route::create(&user, Some(name.to_string()))?;
    } else {
        if name.is_some() {
            println!("⚠️ You only have a single script account. Ignoring name.")
        }
        single_script(&user)?;
        Route::create(&user, None)?;
    }
    println!(
        "✨ Success! Your worker was successfully published. You can view it at {}. ✨",
        user.settings.project.route.unwrap()
    );
    Ok(())
}

fn single_script(user: &User) -> Result<(), failure::Error> {
    let zone_id = &user.settings.project.zone_id;
    let worker_addr = format!(
        "https://api.cloudflare.com/client/v4/zones/{}/workers/script",
        zone_id
    );

    let client = reqwest::Client::new();
    let settings = user.settings.clone();

    client
        .put(&worker_addr)
        .header("X-Auth-Key", settings.global_user.api_key)
        .header("X-Auth-Email", settings.global_user.email)
        .multipart(build_form()?)
        .send()?;

    Ok(())
}

fn multi_script(user: &User, name: &str) -> Result<(), failure::Error> {
    let zone_id = &user.settings.project.zone_id;
    let worker_addr = format!(
        "https://api.cloudflare.com/client/v4/zones/{}/workers/scripts/{}",
        zone_id, name,
    );

    let client = reqwest::Client::new();
    let settings = user.settings.clone();

    client
        .put(&worker_addr)
        .header("X-Auth-Key", settings.global_user.api_key)
        .header("X-Auth-Email", settings.global_user.email)
        .multipart(build_form()?)
        .send()?;

    Ok(())
}

fn build_form() -> Result<Form, failure::Error> {
    let bundle = Bundle::new();

    let form = Form::new()
        .file("metadata", bundle.metadata_path())
        .unwrap_or_else(|_| panic!("{} not found. Did you delete it?", bundle.metadata_path()))
        .file("script", bundle.script_path())
        .unwrap_or_else(|_| {
            panic!(
                "{} not found. Did you rename your js files?",
                bundle.script_path()
            )
        });

    if bundle.has_wasm() {
        Ok(form
            .file("wasmprogram", bundle.wasm_path())
            .unwrap_or_else(|_| {
                panic!(
                    "{} not found. Have you run wrangler build?",
                    bundle.wasm_path()
                )
            }))
    } else {
        Ok(form)
    }
}
