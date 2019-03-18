use std::fs;
use std::path::Path;

use crate::settings::Settings;
use reqwest::multipart::Form;

pub fn publish(zone_id: &str, settings: Settings) -> Result<(), failure::Error> {
    let worker_addr = format!(
        "https://api.cloudflare.com/client/v4/zones/{}/workers/script",
        zone_id
    );

    let client = reqwest::Client::new();

    let res = client
        .put(&worker_addr)
        .header("X-Auth-Key", settings.api_key)
        .header("X-Auth-Email", settings.email)
        .multipart(build_form()?)
        .send();

    println!("{:?}", &res?.text());
    Ok(())
}

fn build_form() -> Result<Form, failure::Error> {
    build_generated_dir()?;
    modify_js()?;
    concat_js()?;
    let form = Form::new()
        .file("metadata", "./worker/metadata_wasm.json")
        .expect("🚧 `./worker/metadata_wasm.json` not found. Did you delete it? 🚧")
        .file("wasmprogram", "./pkg/wasm_worker_bg.wasm")
        .expect("🚧 `./pkg/wasm_worker_bg.wasm` not found. Have you run `wrangler build`? 🚧")
        .file("script", "./worker/generated/script.js")
        .expect("script missing");
    Ok(form)
}

fn build_generated_dir() -> Result<(), failure::Error> {
    let dir = "./worker/generated";
    if !Path::new(dir).is_dir() {
        fs::create_dir("./worker/generated")?;
    }
    Ok(())
}

fn modify_js() -> Result<(), failure::Error> {
    let bindgen_js: String = fs::read_to_string("./pkg/wasm_worker.js")?.parse()?;
    // i am sorry for this hack, plz forgive
    let modded = bindgen_js.replace("module_or_path instanceof WebAssembly.Module", "true");

    fs::write("./worker/generated/wasm_worker.js", modded.as_bytes())?;
    Ok(())
}

fn concat_js() -> Result<(), failure::Error> {
    let bindgen_js: String = fs::read_to_string("./worker/generated/wasm_worker.js")?.parse()?;
    let worker_js: String = fs::read_to_string("./worker/worker.js")?.parse()?;
    let js = format!("{} {}", bindgen_js, worker_js);

    fs::write("./worker/generated/script.js", js.as_bytes())?;
    Ok(())
}
