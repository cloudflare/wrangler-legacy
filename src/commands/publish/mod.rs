mod krate;
pub mod preview;
mod route;
use route::Route;

pub mod package;
use package::Package;

use log::info;

use reqwest::multipart::Form;
use std::fs;
use std::path::Path;

use crate::settings::global_user::GlobalUser;
use crate::settings::project::{Project, ProjectType};
use crate::wranglerjs::Bundle;

pub fn publish(user: GlobalUser, project: Project) -> Result<(), failure::Error> {
    publish_script(&user, &project)?;
    let route = Route::new(&project)?;
    Route::publish(&user, &project, route)?;
    println!(
        "✨ Success! Your worker was successfully published. You can view it at {}. ✨",
        project.route.expect("⚠️ There should be a route")
    );
    Ok(())
}

fn publish_script(user: &GlobalUser, project: &Project) -> Result<(), failure::Error> {
    let worker_addr = format!(
        "https://api.cloudflare.com/client/v4/zones/{}/workers/scripts/{}",
        project.zone_id, project.name,
    );

    let client = reqwest::Client::new();

    let project_type = &project.project_type;
    let mut res = match project_type {
        ProjectType::Rust => {
            info!("Rust project detected. Publishing...");
            client
                .put(&worker_addr)
                .header("X-Auth-Key", &*user.api_key)
                .header("X-Auth-Email", &*user.email)
                .multipart(build_multipart_script()?)
                .send()?
        }
        ProjectType::JavaScript => {
            info!("JavaScript project detected. Publishing...");
            client
                .put(&worker_addr)
                .header("X-Auth-Key", &*user.api_key)
                .header("X-Auth-Email", &*user.email)
                .header("Content-Type", "application/javascript")
                .body(build_js_script()?)
                .send()?
        }
        ProjectType::Webpack => {
            info!("Webpack project detected. Publishing...");
            client
                .put(&worker_addr)
                .header("X-Auth-Key", &*user.api_key)
                .header("X-Auth-Email", &*user.email)
                .multipart(build_webpack_form()?)
                .send()?
        }
    };

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

fn build_js_script() -> Result<String, failure::Error> {
    let package = Package::new("./")?;
    Ok(fs::read_to_string(package.main()?)?)
}

fn build_multipart_script() -> Result<Form, failure::Error> {
    let name = krate::Krate::new("./")?.name.replace("-", "_");
    build_generated_dir()?;
    concat_js(&name)?;

    let metadata_path = "./worker/metadata_wasm.json";
    let wasm_path = &format!("./pkg/{}_bg.wasm", name);
    let script_path = "./worker/generated/script.js";

    Ok(Form::new()
        .file("metadata", metadata_path)
        .unwrap_or_else(|_| panic!("{} not found. Did you delete it?", metadata_path))
        .file("wasmprogram", wasm_path)
        .unwrap_or_else(|_| panic!("{} not found. Have you run wrangler build?", wasm_path))
        .file("script", script_path)
        .unwrap_or_else(|_| panic!("{} not found. Did you rename your js files?", script_path)))
}

fn build_generated_dir() -> Result<(), failure::Error> {
    let dir = "./worker/generated";
    if !Path::new(dir).is_dir() {
        fs::create_dir("./worker/generated")?;
    }
    Ok(())
}

fn concat_js(name: &str) -> Result<(), failure::Error> {
    let bindgen_js_path = format!("./pkg/{}.js", name);
    let bindgen_js: String = fs::read_to_string(bindgen_js_path)?.parse()?;

    let worker_js: String = fs::read_to_string("./worker/worker.js")?.parse()?;
    let js = format!("{} {}", bindgen_js, worker_js);

    fs::write("./worker/generated/script.js", js.as_bytes())?;
    Ok(())
}

fn build_webpack_form() -> Result<Form, failure::Error> {
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
