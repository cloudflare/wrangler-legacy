mod project_assets;
mod wasm_module;

use log::info;

use reqwest::multipart::{Form, Part};
use std::fs;
use std::path::Path;

use crate::commands::build::wranglerjs;
use crate::settings::binding;
use crate::settings::metadata::Metadata;
use crate::settings::project::{Project, ProjectType};

use project_assets::ProjectAssets;
use wasm_module::WasmModule;

use super::{krate, Package};

pub fn build_script_upload_form(project: &Project) -> Result<Form, failure::Error> {
    let project_type = &project.project_type;
    match project_type {
        ProjectType::Rust => {
            info!("Rust project detected. Publishing...");
            let name = krate::Krate::new("./")?.name.replace("-", "_");
            // TODO: move into build?
            build_generated_dir()?;
            concat_js(&name)?;

            let path = format!("./pkg/{}_bg.wasm", name).to_string();
            let binding = "wasm".to_string();
            let wasm_module = WasmModule::new(path, binding)?;

            let script_path = "./worker/generated/script.js".to_string();

            let assets = ProjectAssets::new(script_path, vec![wasm_module])?;

            build_form(&assets)
        }
        ProjectType::JavaScript => {
            info!("JavaScript project detected. Publishing...");
            let package = Package::new("./")?;

            let script_path = package.main()?;

            let assets = ProjectAssets::new(script_path, Vec::new())?;

            build_form(&assets)
        }
        ProjectType::Webpack => {
            info!("Webpack project detected. Publishing...");
            // FIXME(sven): shouldn't new
            let bundle = wranglerjs::Bundle::new();

            let script_path = bundle.script_path();

            let mut wasm_modules = Vec::new();

            if bundle.has_wasm() {
                let path = bundle.wasm_path();
                let binding = bundle.get_wasm_binding();
                let wasm_module = WasmModule::new(path, binding)?;
                wasm_modules.push(wasm_module)
            }

            let assets = ProjectAssets::new(script_path, wasm_modules)?;

            build_form(&assets)
        }
    }
}

fn build_form(assets: &ProjectAssets) -> Result<Form, failure::Error> {
    let mut form = Form::new();

    // The preview service in particular streams the request form, and requires that the
    // "metadata" part be set first, so this order is important.
    form = add_metadata(form, assets)?;
    form = add_files(form, assets)?;

    Ok(form)
}

fn add_files(mut form: Form, assets: &ProjectAssets) -> Result<Form, failure::Error> {
    form = form.file(assets.script_name(), assets.script_path())?;

    for wasm_module in &assets.wasm_modules {
        form = form.file(wasm_module.filename(), wasm_module.path())?;
    }

    Ok(form)
}

fn add_metadata(mut form: Form, assets: &ProjectAssets) -> Result<Form, failure::Error> {
    let metadata_json = serde_json::json!(&Metadata {
        body_part: assets.script_name(),
        bindings: assets.bindings(),
    });

    let metadata = Part::text((metadata_json).to_string())
        .file_name("metadata.json")
        .mime_str("application/json")?;

    form = form.part("metadata", metadata);

    Ok(form)
}

fn filename_from_path(path: &String) -> Option<String> {
    let path = Path::new(path);
    path.file_stem()?.to_str().map(|s| s.to_string())
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
