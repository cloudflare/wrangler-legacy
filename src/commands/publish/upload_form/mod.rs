mod project_assets;
mod text_blob;
mod wasm_module;

use reqwest::multipart::{Form, Part};
use std::fs;
use std::path::Path;

use crate::commands;
use crate::commands::build::wranglerjs;
use crate::commands::kv::bucket::AssetManifest;
use crate::settings::binding;
use crate::settings::metadata::Metadata;
use crate::settings::target::{Target, TargetType};

use project_assets::ProjectAssets;
use text_blob::TextBlob;
use wasm_module::WasmModule;

use super::{krate, Package};

pub fn build_script_and_upload_form(
    target: &Target,
    asset_manifest: Option<AssetManifest>,
) -> Result<Form, failure::Error> {
    // Build the script before uploading.
    commands::build(&target)?;

    let target_type = &target.target_type;
    let kv_namespaces = target.kv_namespaces();
    match target_type {
        TargetType::Rust => {
            log::info!("Rust project detected. Publishing...");
            let name = krate::Krate::new("./")?.name.replace("-", "_");
            // TODO: move into build?
            build_generated_dir()?;
            concat_js(&name)?;

            let path = format!("./pkg/{}_bg.wasm", name);
            let binding = "wasm".to_string();
            let wasm_module = WasmModule::new(path, binding)?;

            let script_path = "./worker/generated/script.js".to_string();

            let assets =
                ProjectAssets::new(script_path, vec![wasm_module], kv_namespaces, Vec::new())?;

            build_form(&assets)
        }
        TargetType::JavaScript => {
            log::info!("JavaScript project detected. Publishing...");
            let build_dir = target.build_dir()?;
            let package = Package::new(&build_dir)?;

            let script_path = package.main(&build_dir)?;

            let assets = ProjectAssets::new(script_path, Vec::new(), kv_namespaces, Vec::new())?;

            build_form(&assets)
        }
        TargetType::Webpack => {
            log::info!("Webpack project detected. Publishing...");
            // FIXME(sven): shouldn't new
            let build_dir = target.build_dir()?;
            let bundle = wranglerjs::Bundle::new(&build_dir);

            let script_path = bundle.script_path();

            let mut wasm_modules = Vec::new();

            if bundle.has_wasm() {
                let path = bundle.wasm_path();
                let binding = bundle.get_wasm_binding();
                let wasm_module = WasmModule::new(path, binding)?;
                wasm_modules.push(wasm_module);
            }

            let mut text_blobs = Vec::new();

            if let Some(asset_manifest) = asset_manifest {
                log::info!("adding __STATIC_CONTENT_MANIFEST");
                let binding = "__STATIC_CONTENT_MANIFEST".to_string();
                let asset_manifest_blob = get_asset_manifest_blob(asset_manifest)?;
                let text_blob = TextBlob::new(asset_manifest_blob, binding)?;
                text_blobs.push(text_blob);
            }

            let assets = ProjectAssets::new(script_path, wasm_modules, kv_namespaces, text_blobs)?;

            build_form(&assets)
        }
    }
}

fn get_asset_manifest_blob(asset_manifest: AssetManifest) -> Result<String, failure::Error> {
    let asset_manifest = serde_json::to_string(&asset_manifest)?;
    Ok(asset_manifest)
}

fn build_form(assets: &ProjectAssets) -> Result<Form, failure::Error> {
    let mut form = Form::new();

    // The preview service in particular streams the request form, and requires that the
    // "metadata" part be set first, so this order is important.
    form = add_metadata(form, assets)?;
    form = add_files(form, assets)?;

    log::info!("building form");
    log::info!("{:?}", &form);

    Ok(form)
}

fn add_files(mut form: Form, assets: &ProjectAssets) -> Result<Form, failure::Error> {
    form = form.file(assets.script_name(), assets.script_path())?;

    for wasm_module in &assets.wasm_modules {
        form = form.file(wasm_module.filename(), wasm_module.path())?;
    }

    for text_blob in &assets.text_blobs {
        let part = Part::text(text_blob.data.clone())
            .file_name(text_blob.binding.clone())
            .mime_str("text/plain")?;

        form = form.part(text_blob.binding.clone(), part);
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

fn filename_from_path(path: &str) -> Option<String> {
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
