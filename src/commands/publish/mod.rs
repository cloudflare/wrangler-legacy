mod krate;
pub mod preview;
mod route;
use route::Route;

use std::fs;
use std::path::Path;

use crate::user::User;

use failure::ResultExt;
use reqwest::multipart::Form;

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
    let name = krate::Krate::new("./")?.name.replace("-", "_");
    build_generated_dir()?;
    modify_js(&name)?;
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

fn modify_js(name: &str) -> Result<(), failure::Error> {
    let bindgen_js_path = format!("./pkg/{}.js", name);
    let bindgen_js: String = fs::read_to_string(&bindgen_js_path)
        .context(format!("Error reading {}", bindgen_js_path))?
        .parse()?;
    // i am sorry for this hack, plz forgive
    let modded = bindgen_js.replace("module_or_path instanceof WebAssembly.Module", "true");

    let modded_js_path = format!("./worker/generated/{}.js", name);
    fs::write(modded_js_path, modded.as_bytes())?;
    Ok(())
}

fn concat_js(name: &str) -> Result<(), failure::Error> {
    let bindgen_js_path = format!("./worker/generated/{}.js", name);
    let bindgen_js: String = fs::read_to_string(bindgen_js_path)?.parse()?;

    let worker_js: String = fs::read_to_string("./worker/worker.js")?.parse()?;
    let js = format!("{} {}", bindgen_js, worker_js);

    fs::write("./worker/generated/script.js", js.as_bytes())?;
    Ok(())
}
