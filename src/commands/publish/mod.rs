mod krate;
pub mod preview;
mod route;
use route::Route;

pub mod package;
use package::Package;

use log::info;

use reqwest::multipart::Form;
use std::collections::HashMap;
use std::fs;
use std::path::Path;

use crate::commands::build::wranglerjs::Bundle;
use crate::commands::subdomain::Subdomain;
use crate::http;
use crate::settings::global_user::GlobalUser;
use crate::settings::project::{Project, ProjectType};
use crate::terminal::message;

pub fn publish(user: &GlobalUser, project: &Project, release: bool) -> Result<(), failure::Error> {
    info!("release = {}", release);

    let missing_fields = match (project.account_id.is_empty(), project.name.is_empty()) {
        (true, true) => Some("an account_id and name"),
        (true, false) => Some("an account_id"),
        (false, true) => Some("a name"),
        (false, false) => None,
    };

    //perform extra checks for route and zone_id for a publish --release
    let missing_fields_release = match (release, &project.zone_id, &project.route) {
        (true, None, None) => Some("a zone_id and route"),
        (true, None, Some(_)) => Some("a zone_id"),
        (true, Some(_), None) => Some("a route"),
        (_, _, _) => None,
    };

    if let Some(mf) = missing_fields {
        failure::bail!(
            "Check your wrangler.toml, it is missing {} which is required to publish!",
            mf
        )
    }

    if let Some(mf) = missing_fields_release {
        failure::bail!(
            "Check your wrangler.toml, it is missing {} which is required to publish a release!",
            mf
        )
    }

    create_kv_namespaces(user, project)?;
    publish_script(&user, &project, release)?;
    if release {
        info!("release mode detected, making a route...");
        let route = Route::new(&project)?;
        Route::publish(&user, &project, &route)?;
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

pub fn create_kv_namespaces(user: &GlobalUser, project: &Project) -> Result<(), failure::Error> {
    let kv_addr = format!(
        "https://api.cloudflare.com/client/v4/accounts/{}/storage/kv/namespaces",
        project.account_id,
    );

    let client = http::client();

    if let Some(namespaces) = &project.kv_namespaces {
        for namespace in namespaces {
            info!("Attempting to create namespace '{}'", namespace);

            let mut map = HashMap::new();
            map.insert("title", namespace);

            let request = client
                .post(&kv_addr)
                .header("X-Auth-Key", &*user.api_key)
                .header("X-Auth-Email", &*user.email)
                .json(&map)
                .send();

            if let Err(error) = request {
                // A 400 is returned if the account already owns a namespace with this title.
                //
                // https://api.cloudflare.com/#workers-kv-namespace-create-a-namespace
                match error.status() {
                    Some(code) if code == 400 => {
                        info!("Namespace '{}' already exists, continuing.", namespace)
                    }
                    _ => {
                        info!("Error when creating namespace '{}'", namespace);
                        failure::bail!("⛔ Something went wrong! Error: {}", error)
                    }
                }
            }
            info!("Namespace '{}' exists now", namespace)
        }
    }
    Ok(())
}

fn publish_script(
    user: &GlobalUser,
    project: &Project,
    release: bool,
) -> Result<(), failure::Error> {
    let worker_addr = format!(
        "https://api.cloudflare.com/client/v4/accounts/{}/workers/scripts/{}",
        project.account_id, project.name,
    );

    let client = http::client();

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
        message::success("Successfully published your script.");
    } else {
        failure::bail!(
            "Something went wrong! Status: {}, Details {}",
            res.status(),
            res.text()?
        )
    }

    if !release {
        let private = project.private.unwrap_or(false);
        if !private {
            info!("--release not passed, publishing to subdomain");
            make_public_on_subdomain(project, user)?;
        }
    }

    Ok(())
}

fn build_subdomain_request() -> String {
    serde_json::json!({ "enabled":true}).to_string()
}

fn make_public_on_subdomain(project: &Project, user: &GlobalUser) -> Result<(), failure::Error> {
    info!("checking that subdomain is registered");
    let subdomain = Subdomain::get(&project.account_id, user)?;

    let sd_worker_addr = format!(
        "https://api.cloudflare.com/client/v4/accounts/{}/workers/scripts/{}/subdomain",
        project.account_id, project.name,
    );

    let client = http::client();

    info!("Making public on subdomain...");
    let mut res = client
        .post(&sd_worker_addr)
        .header("X-Auth-Key", &*user.api_key)
        .header("X-Auth-Email", &*user.email)
        .header("Content-type", "application/json")
        .body(build_subdomain_request())
        .send()?;

    if res.status().is_success() {
        let msg = format!(
            "Successfully made your script available at https://{}.{}.workers.dev",
            project.name, subdomain
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
