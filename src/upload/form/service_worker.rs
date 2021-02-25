use reqwest::blocking::multipart::{Form, Part};
use serde::Serialize;

use crate::settings::binding::Binding;

use super::ServiceWorkerAssets;

#[derive(Serialize, Debug)]
struct Metadata {
    pub body_part: String,
    pub bindings: Vec<Binding>,
}

pub fn build_form(
    assets: &ServiceWorkerAssets,
    session_config: Option<serde_json::Value>,
) -> Result<Form, failure::Error> {
    let mut form = Form::new();

    // The preview service in particular streams the request form, and requires that the
    // "metadata" part be set first, so this order is important.
    form = add_metadata(form, assets)?;
    form = add_files(form, assets)?;
    if let Some(session_config) = session_config {
        form = add_session_config(form, session_config)?
    }

    log::info!("building form");
    log::info!("{:#?}", &form);

    Ok(form)
}

fn add_files(mut form: Form, assets: &ServiceWorkerAssets) -> Result<Form, failure::Error> {
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

fn add_metadata(mut form: Form, assets: &ServiceWorkerAssets) -> Result<Form, failure::Error> {
    let metadata_json = serde_json::json!(&Metadata {
        body_part: assets.script_name(),
        bindings: assets.bindings(),
    });

    let metadata = Part::text(metadata_json.to_string())
        .file_name("metadata.json")
        .mime_str("application/json")?;

    form = form.part("metadata", metadata);

    Ok(form)
}

fn add_session_config(
    mut form: Form,
    session_config: serde_json::Value,
) -> Result<Form, failure::Error> {
    let wrangler_session_config = Part::text(session_config.to_string())
        .file_name("")
        .mime_str("application/json")?;

    form = form.part("wrangler-session-config", wrangler_session_config);

    Ok(form)
}
