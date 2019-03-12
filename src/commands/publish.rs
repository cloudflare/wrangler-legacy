use std::fs;
use std::io::Write;

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
        .header("X-Auth-Key", settings.auth_key)
        .header("X-Auth-Email", settings.email)
        .multipart(build_form()?)
        .send();

    println!("{:?}", &res?.text());
    Ok(())
}

fn build_form() -> Result<Form, failure::Error> {
    modify_js()?;
    concat_js()?;
    let form = Form::new()
        .file("metadata", "./worker/metadata_wasm.json")
        .expect("metadata missin")
        .file("wasmprogram", "./pkg/wasm_worker_bg.wasm")
        .expect("wasm missing")
        .file("script", "./worker/script.js")
        .expect("script missing");
    Ok(form)
}

fn modify_js() -> Result<(), failure::Error> {
    let bindgen_js: String = fs::read_to_string("./pkg/wasm_worker.js")?.parse()?;
    // i am sorry for this hack, plz forgive
    let modded = bindgen_js.replace("path_or_module instanceof WebAssembly.Module", "true");

    let mut modified_bindgen_js = fs::File::create("./worker/wasm_worker.js")?;
    modified_bindgen_js.write_all(modded.as_bytes())?;
    Ok(())
}

fn concat_js() -> Result<(), failure::Error> {
    let bindgen_js: String = fs::read_to_string("./worker/wasm_worker.js")?.parse()?;
    let worker_js: String = fs::read_to_string("./worker/worker.js")?.parse()?;
    let js = format!("{} {}", bindgen_js, worker_js);

    let mut script = fs::File::create("./worker/script.js")?;
    script.write_all(js.as_bytes())?;
    Ok(())
}
