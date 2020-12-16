use std::fs::File;

use reqwest::blocking::multipart::{Form, Part};
use serde::Serialize;

use crate::settings::binding::Binding;

use super::ModulesAssets;

#[derive(Serialize, Debug)]
struct Metadata {
    pub main_module: String,
    pub bindings: Vec<Binding>,
}

pub fn build_form(
    assets: &ModulesAssets,
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

fn add_files(mut form: Form, assets: &ModulesAssets) -> Result<Form, failure::Error> {
    for module in &assets.modules {
        let file_name = module
            .filename()
            .ok_or_else(|| failure::err_msg("a filename is required for each module"))?;
        let part = Part::reader(File::open(module.path.clone())?)
            .mime_str(module.module_type.content_type())?
            .file_name(file_name.clone());
        form = form.part(file_name.clone(), part);
    }
    Ok(form)
}

fn add_metadata(mut form: Form, assets: &ModulesAssets) -> Result<Form, failure::Error> {
    let metadata_json = serde_json::json!(&Metadata {
        main_module: assets.main_module.clone(),
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
