use log::info;

use reqwest::multipart::{Form, Part};
use std::fs;
use std::path::Path;

use crate::commands::build::wranglerjs::Bundle;
use crate::settings::binding::Binding;
use crate::settings::metadata::Metadata;
use crate::settings::project::{Project, ProjectType};

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

            let wasm_module = WasmModule {
                path: format!("./pkg/{}_bg.wasm", name).to_string(),
                filename: "wasm".to_string(),
                binding: "wasmprogram".to_string(),
            };

            let script_path = "./worker/generated/script.js".to_string();

            build_form(script_path, Some(wasm_module))
        }
        ProjectType::JavaScript => {
            info!("JavaScript project detected. Publishing...");
            let package = Package::new("./")?;
            let script_path = package.main()?;

            build_form(script_path, None)
        }
        ProjectType::Webpack => {
            info!("Webpack project detected. Publishing...");
            // FIXME(sven): shouldn't new
            let bundle = Bundle::new();

            let script_path = bundle.script_path();

            let wasm_module = WasmModule {
                path: bundle.wasm_path(),
                filename: bundle.get_wasm_binding(),
                binding: bundle.get_wasm_binding(),
            };

            build_form(script_path, Some(wasm_module))
        }
    }
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

#[derive(Debug)]
struct WasmModule {
    path: String,
    filename: String,
    binding: String,
}

impl ToBinding for WasmModule {
    fn to_binding(&self) -> Binding {
        let name = self.filename.clone();
        let part = self.binding.clone();
        Binding::new_wasm_module(name, part)
    }
}

trait ToBinding {
    fn to_binding(&self) -> Binding;
}

fn build_form(
    script_path: String,
    wasm_module: Option<WasmModule>,
) -> Result<Form, failure::Error> {
    match wasm_module {
        Some(wasm) => {
            let bindings = vec![wasm.to_binding()];
            let wasm_filename = wasm.filename.clone();

            let metadata_json = generate_metadata_json(bindings);

            let metadata = Part::text((metadata_json).to_string())
                .file_name("metadata.json")
                .mime_str("application/json")?;

            Ok(Form::new()
                .part("metadata", metadata)
                .file("script", &script_path)
                .unwrap_or_else(|_| {
                    panic!("{} not found. Did you rename your js files?", &script_path)
                })
                .file(wasm_filename, &wasm.path)
                .unwrap_or_else(|_| {
                    panic!("{} not found. Have you run wrangler build?", &wasm.path)
                }))
        }
        None => {
            let bindings = vec![];
            let metadata_json = generate_metadata_json(bindings);

            let metadata = Part::text((metadata_json).to_string())
                .file_name("metadata.json")
                .mime_str("application/json")?;

            Ok(Form::new()
                .part("metadata", metadata)
                .file("script", &script_path)
                .unwrap_or_else(|_| {
                    panic!("{} not found. Did you rename your js files?", &script_path)
                }))
        }
    }
}

fn generate_metadata_json(bindings: Vec<Binding>) -> serde_json::value::Value {
    serde_json::json!(&Metadata {
        body_part: "script".to_string(),
        bindings,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use assert_json_diff::assert_json_eq;
    use serde_json::json;

    #[test]
    fn rust_wasm_generates_same_metadata() {
        let wasm_module = Binding::new_wasm_module("wasm".to_string(), "wasmprogram".to_string());
        let bindings = vec![wasm_module];

        let expected_json = json!({
            "body_part": "script",
            "bindings": [
                {
                    "name": "wasm",
                    "type": "wasm_module",
                    "part": "wasmprogram"
                }
            ]
        });

        let actual_json = generate_metadata_json(bindings);

        assert_json_eq!(actual_json, expected_json);
    }

    #[test]
    fn webpack_with_wasm_generates_same_metadata() {
        let wasm_module =
            Binding::new_wasm_module("wasmprogram".to_string(), "wasmprogram".to_string());
        let bindings = vec![wasm_module];
        let expected_json = json!({
            "body_part": "script",
            "bindings": [
                {
                    "type":"wasm_module",
                    "name":"wasmprogram",
                    "part":"wasmprogram"
                }
            ]
        });

        let actual_json = generate_metadata_json(bindings);

        assert_json_eq!(actual_json, expected_json);
    }

    #[test]
    fn webpack_without_wasm_generates_same_metadata() {
        let bindings = Vec::new();
        let expected_json = json!({
            "body_part": "script",
            "bindings": []
        });

        let actual_json = generate_metadata_json(bindings);

        assert_json_eq!(actual_json, expected_json);
    }
}
