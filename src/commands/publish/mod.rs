mod krate;
pub mod preview;
mod route;
use route::Route;

use log::info;

use reqwest::multipart::Form;
use std::fs;

use crate::user::settings::ProjectType;
use crate::user::User;
use crate::wranglerjs::Bundle;

pub fn publish(user: User) -> Result<(), failure::Error> {
    let name = &user.settings.project.name;
    multi_script(&user, name)?;
    Route::create(&user, Some(name.to_string()))?;
    println!(
        "✨ Success! Your worker was successfully published. You can view it at {}. ✨",
        user.settings
            .project
            .route
            .expect("⚠️ There should be a route")
    );
    Ok(())
}

fn multi_script(user: &User, name: &str) -> Result<(), failure::Error> {
    let zone_id = &user.settings.project.zone_id;
    let project_type = &user.settings.project.project_type;
    let worker_addr = format!(
        "https://api.cloudflare.com/client/v4/zones/{}/workers/scripts/{}",
        zone_id, name,
    );

    let client = reqwest::Client::new();
    let settings = user.settings.clone();

    let mut res = client
        .put(&worker_addr)
        .header("X-Auth-Key", settings.global_user.api_key)
        .header("X-Auth-Email", settings.global_user.email)
        .multipart(build_form()?)
        .send()?;

    if res.status().is_success() {
        println!("🥳 Successfully published your script.")
    } else {
        failure::bail!(
            "⛔ Something went wrong! Status: {}, Details {}",
            res.status(),
            res.text()?
        )
    }

    Ok(())
}

fn build_js() -> Result<String, failure::Error> {
    Ok(fs::read_to_string("worker.js")?)
}

fn build_form() -> Result<Form, failure::Error> {
    // FIXME(sven): shouldn't new
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
        println!("add wasm bindinds");
        Ok(form
            .file(bundle.get_wasm_binding(), bundle.wasm_path())
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
