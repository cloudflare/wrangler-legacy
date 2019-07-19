mod file;
mod project_assets;
mod wasm_module;

use log::info;

use reqwest::multipart::{Form, Part};
use std::fs;
use std::path::Path;

use crate::commands::build::wranglerjs::Bundle;
use crate::settings::binding;
use crate::settings::metadata::Metadata;
use crate::settings::project::kv_namespace;
use crate::settings::project::{Project, ProjectType};

use project_assets::ProjectAssets;
use wasm_module::WasmModule;

use super::{krate, Package};

pub fn build_script_upload_form(project: &Project) -> Result<Form, failure::Error> {
    let project_type = &project.project_type;
    let kv_namespaces = project.kv_namespaces();
    match project_type {
        ProjectType::Rust => {
            info!("Rust project detected. Publishing...");
            let name = krate::Krate::new("./")?.name.replace("-", "_");
            // TODO: move into build?
            build_generated_dir()?;
            concat_js(&name)?;

            let wasm_module = WasmModule {
                path: format!("./pkg/{}_bg.wasm", name).to_string(),
                filename: "wasmprogram".to_string(),
                binding: "wasm".to_string(),
            };

            let script_path = "./worker/generated/script.js".to_string();

            let assets = ProjectAssets {
                script_path,
                wasm_modules: vec![wasm_module],
                kv_namespaces,
            };

            build_form(&assets)
        }
        ProjectType::JavaScript => {
            info!("JavaScript project detected. Publishing...");
            let package = Package::new("./")?;

            let script_path = package.main()?;

            let assets = ProjectAssets {
                script_path,
                wasm_modules: Vec::new(),
                kv_namespaces,
            };

            build_form(&assets)
        }
        ProjectType::Webpack => {
            info!("Webpack project detected. Publishing...");
            // FIXME(sven): shouldn't new
            let bundle = Bundle::new();

            let script_path = bundle.script_path();

            let mut wasm_modules = Vec::new();

            if bundle.has_wasm() {
                let wasm_module = WasmModule {
                    path: bundle.wasm_path(),
                    filename: bundle.get_wasm_binding(),
                    binding: bundle.get_wasm_binding(),
                };
                wasm_modules.push(wasm_module)
            }

            let assets = ProjectAssets {
                script_path,
                wasm_modules,
                kv_namespaces,
            };

            build_form(&assets)
        }
    }
}

fn build_form(assets: &ProjectAssets) -> Result<Form, failure::Error> {
    let mut form = Form::new();

    form = add_files(form, assets)?;
    form = add_metadata(form, assets)?;

    Ok(form)
}

fn add_files(mut form: Form, assets: &ProjectAssets) -> Result<Form, failure::Error> {
    for file in assets.files() {
        form = form.file(file.name, file.path)?;
    }

    Ok(form)
}

fn add_metadata(mut form: Form, assets: &ProjectAssets) -> Result<Form, failure::Error> {
    let metadata_json = serde_json::json!(&Metadata {
        body_part: "script".to_string(),
        bindings: assets.bindings(),
    });

    let metadata = Part::text((metadata_json).to_string())
        .file_name("metadata.json")
        .mime_str("application/json")?;

    form = form.part("metadata", metadata);

    Ok(form)
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
