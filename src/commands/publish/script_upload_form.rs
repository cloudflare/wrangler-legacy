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
    let script_upload_form = match project_type {
        ProjectType::Rust => {
            info!("Rust project detected. Publishing...");
            build_multipart_script()?
        }
        ProjectType::JavaScript => {
            info!("JavaScript project detected. Publishing...");
            build_js_script()?
        }
        ProjectType::Webpack => {
            info!("Webpack project detected. Publishing...");
            build_webpack_form()?
        }
    };

    Ok(script_upload_form)
}

fn build_js_script() -> Result<Form, failure::Error> {
    let package = Package::new("./")?;
    let script_path = package.main()?;
    let metadata_json = r#"{"body_part":"script","bindings":[]}"#;

    let metadata = Part::text(metadata_json)
        .file_name("metadata.json")
        .mime_str("application/json")?;

    Ok(Form::new()
        .file("script", &script_path)
        .unwrap_or_else(|_| panic!("{} not found. Did you rename your js files?", &script_path))
        .part("metadata", metadata))
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
